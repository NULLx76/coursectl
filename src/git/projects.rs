use std::vec;

use color_eyre::{
    eyre::{eyre, Context, Result},
    Section,
};
use gitlab::{
    api::{
        groups::projects::GroupProjects,
        ignore, paged,
        projects::protected_branches::{ProtectedBranches, UnprotectBranch},
        Client, Query, RestClient,
    },
    Gitlab,
};
use indicatif::ProgressIterator;
use serde::Deserialize;

use crate::models::{GitlabApiResponse, ProjectInfo};
use http::request::Builder as RequestBuilder;

pub(crate) fn get_projects_by_group(client: &Gitlab, id: u64) -> Result<Vec<ProjectInfo>> {
    let endpoint = GroupProjects::builder().group(id).archived(false).build()?;

    Ok(paged(endpoint, gitlab::api::Pagination::All).query(client)?)
}

pub fn list(client: &Gitlab, id: u64) -> Result<()> {
    let projects = get_projects_by_group(client, id)?;

    for project in projects {
        let name = project.name.replace(' ', "-").to_lowercase();
        println!("{name} {}", project.ssh_url_to_repo);
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Branch {
    name: String,
}

pub fn unprotect(client: &Gitlab, group: u64, branch: &str, dry_run: bool) -> Result<()> {
    let projects = get_projects_by_group(client, group)?;
    let mut n = 0;

    for project in projects.into_iter().progress() {
        let endpoint = ProtectedBranches::builder()
            .project(project.id.value())
            .build()?;

        let branches: Vec<Branch> = endpoint.query(client)?;

        if branches.iter().any(|b| b.name == branch) {
            if dry_run {
                println!("Dry Run: unprotected {branch} on {}", project.name);
            } else {
                let endpoint = UnprotectBranch::builder()
                    .project(project.id.value())
                    .name(branch)
                    .build()?;

                ignore(endpoint).query(client)?;
            }

            n += 1;
        }
    }
    println!("Unprotected {branch} on {n} projects successfully");

    Ok(())
}

pub fn unfork(client: &Gitlab, group: u64, dry_run: bool) -> Result<()> {
    let projects = get_projects_by_group(client, group)?;

    let mut errors = vec![];

    for project in projects.into_iter().progress() {
        let id = project.id;

        if dry_run {
            println!("Unforking project {id}");
            continue;
        }

        let endpoint = client
            .rest_endpoint(&format!("/api/v4/projects/{id}/fork"))
            .wrap_err("could not build endpoint")?;

        let req = RequestBuilder::new()
            .method("DELETE")
            .uri(endpoint.as_ref());

        // 204 on success
        // 304 if not modified

        match client.rest(req, vec![]).wrap_err("unforking error") {
            Ok(rsp) => {
                let status = rsp.status();

                let v: GitlabApiResponse = if let Ok(v) = serde_json::from_slice(rsp.body()) {
                    v
                } else {
                    if status.is_success() {
                        continue;
                    }

                    errors
                        .push(Err(eyre!("unfork server error {status}")
                            .section(format!("{:?}", rsp.body()))));
                    continue;
                };

                if !status.is_success() {
                    errors.push(Err(
                        eyre!("unfork error").section(format!("{:?}", v.message))
                    ));
                    continue;
                }

                if v.status != "success" {
                    eprintln!("git unfork error: {:?}", v.message);
                }
            }
            e@Err(_) => errors.push(e)
        }
    }

    if !errors.is_empty() {
        eprintln!("The following errors occured while unforking: {:#?}", errors);
    }

    Ok(())
}
