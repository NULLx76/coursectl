use crate::git::invite;
use crate::{
    brightspace::get_students,
    models::{Group, ProjectInfo, Student},
};
use color_eyre::eyre::{Context, Result};
use gitlab::api::common::VisibilityLevel;
use gitlab::{
    api::{
        common::AccessLevel,
        projects::{self},
        Query,
    },
    Gitlab,
};
use http::Uri;
use indicatif::ProgressIterator;
use itertools::Itertools;

// Creates Gitlab Repos inviting all group members
pub fn create_group_repos(
    client: &Gitlab,
    parent_namespace_id: u64,
    template_url: &str,
    access_level: AccessLevel,
    groups: &[Group],
    dry_run: bool,
) -> Result<()> {
    let mut n = 0;
    let mut created = Vec::new();
    for g in groups.iter().progress() {
        if dry_run {
            created.push(g);
        } else {
            create_repo_from_template(
                client,
                g.members.iter().collect_vec().as_ref(),
                parent_namespace_id,
                &g.name,
                template_url,
                access_level,
            )
            .wrap_err(format!("failed creating repo for: {}", g.name))?;
        }

        n += 1;
    }

    println!("Created {n} projects successfully.");
    if dry_run {
        println!("Would have created repo for: {created:#?}");
    }
    Ok(())
}

/// TODO: Pull out brightspace code
#[allow(clippy::too_many_arguments)]
pub fn create_individual_repos(
    client: &Gitlab,
    repo_name_prefix: &Option<String>,
    parent_namespace_id: u64,
    template_url: &str,
    access_level: AccessLevel,
    brightspace_cookie: &str,
    brightspace_base_url: &Uri,
    brightspace_ou: u64,
    dry_run: bool,
) -> Result<()> {
    let students = get_students(brightspace_base_url, brightspace_cookie, brightspace_ou)
        .wrap_err("failed getting list of students from brightspace")?;

    let parent_project_names: Vec<String> =
        crate::projects::get_projects_by_group(client, parent_namespace_id)
            .wrap_err("failed getting projects under give parent id")?
            .into_iter()
            .map(|p| p.name)
            .collect();

    let mut n = 0;
    let mut skipped = 0;
    let mut created = Vec::new();

    for s in students.into_iter().progress() {
        let name = if let Some(prefix) = &repo_name_prefix {
            format!("{prefix} - {}", &s.netid)
        } else {
            s.netid.clone()
        };

        if parent_project_names.iter().any(|pn| pn == &name) {
            // println!("Skipping {}, already has a repo.", &s.netid);
            skipped += 1;
            continue;
        }

        if dry_run {
            created.push(s);
        } else {
            create_repo_from_template(
                client,
                &[&s],
                parent_namespace_id,
                &name,
                template_url,
                access_level,
            )
            .wrap_err("failed creating repo")?;
        }

        n += 1;
    }

    println!("Created {n} projects successfully, skipped {skipped} students.");
    if dry_run {
        println!("Would have created repo for: {created:?}");
    }

    Ok(())
}

fn create_repo_from_template(
    client: &Gitlab,
    students: &[&Student],
    parent_namespace_id: u64,
    name: &str,
    template_url: &str,
    access_level: AccessLevel,
) -> Result<()> {
    let endpoint = projects::CreateProject::builder()
        .visibility(VisibilityLevel::Private)
        .import_url(template_url)
        .namespace_id(parent_namespace_id)
        .name(name)
        .emails_disabled(true)
        .build()
        .wrap_err("createproject builder")?;

    let project: ProjectInfo = endpoint.query(client).wrap_err("create project")?;

    invite::add_students_to_project(client, project.id, students, access_level)
}

fn fork_template(
    client: &Gitlab,
    students: &[&Student],
    group_id: u64,
    name: &str,
    template_path: &str,
    access_level: AccessLevel,
) -> Result<()> {
    let endpoint = projects::ForkProject::builder()
        .visibility(VisibilityLevel::Private)
        .project(template_path)
        .namespace_id(group_id)
        .name(name)
        .path(name)
        .build()
        .wrap_err("fork builder")?;

    let project: ProjectInfo = endpoint.query(client).wrap_err("fork project")?;

    invite::add_students_to_project(client, project.id, students, access_level)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use gitlab::Gitlab;

    use super::fork_template;

    #[test]
    #[ignore = "flaky"]
    fn test_fork() {
        dotenv::dotenv().ok();
        let token = env::var("GITLAB_COM_TOKEN").unwrap();

        let client = Gitlab::new("gitlab.com", token).unwrap();

        fork_template(
            &client,
            &[],
            60104344,
            "Test-666",
            "0x76/multi-branch-test-project",
            gitlab::api::common::AccessLevel::Developer,
        )
        .unwrap();
    }
}
