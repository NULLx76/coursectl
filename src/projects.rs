use gitlab::{
    api::{
        groups::projects::GroupProjects, ignore, paged,
        projects::protected_branches::UnprotectBranch, Query,
    },
    Gitlab, ProjectId,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    id: ProjectId,
    name: String,
    ssh_url_to_repo: String,
}

fn get_projects_by_group(client: &Gitlab, id: u64) -> Vec<Project> {
    let endpoint = GroupProjects::builder()
        .group(id)
        .archived(false)
        .build()
        .unwrap();

    paged(endpoint, gitlab::api::Pagination::All)
        .query(client)
        .unwrap()
}

pub fn list(client: &Gitlab, id: u64) {
    let projects = get_projects_by_group(client, id);

    for project in projects {
        let name = project.name.replace(' ', "-").to_lowercase();
        println!("{name} {}", project.ssh_url_to_repo);
    }
}

pub fn unprotect(client: &Gitlab, group: u64, branch: &str) {
    let projects = get_projects_by_group(client, group);
    let n = projects.len();

    for project in projects {
        let endpoint = UnprotectBranch::builder()
            .project(project.id.value())
            .name(branch)
            .build()
            .unwrap();

        // TODO: Ignore if branch already unprotected / doesn't exist
        ignore(endpoint).query(client).unwrap();
    }
    println!("Unprotected {branch} on {n} projects successfully");
}
