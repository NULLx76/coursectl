

use color_eyre::{
    eyre::{Result},
};
use gitlab::{api::{
    groups::projects::GroupProjects,
    ignore, paged,
    projects::{protected_branches::{ProtectedBranches, UnprotectBranch}}, Query,
}, Gitlab};
use gitlab::api::ApiError;
use indicatif::ProgressIterator;
use serde::Deserialize;

use crate::models::ProjectInfo;

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

// pub fn unfork(client: &Gitlab, group: u64, dry_run: bool) -> Result<()> {
//     let projects = get_projects_by_group(client, group)?;

//     for project in projects.into_iter().progress() {
//         if dry_run {
//             println!("Dry Run: Unforking project {}", project.id.value());
//             continue;
//         }

//         let endpoint = UnforkProject::builder().project(project.id.value()).build()?;

//         match ignore(endpoint).query(client) {
//             Ok(_) => {}
//             Err(ApiError::GitlabService { status, ..}) if status.as_u16() == 304 => {
//                 // not a fork
//             }
//             e@Err(_) => e?,
//         }
//     }

//     Ok(())
// }