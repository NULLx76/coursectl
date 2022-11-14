use gitlab::{
    api::{groups::projects::GroupProjects, Query},
    Gitlab,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
    ssh_url_to_repo: String,
}

pub fn list_projects(client: &Gitlab, id: u64) {
    let endpoint = GroupProjects::builder().group(id).build().unwrap();

    let projects: Vec<Project> = endpoint.query(client).unwrap();
    for project in projects {
        let name = project.name.replace(' ', "-").to_lowercase();
        println!("{name} {}", project.ssh_url_to_repo);
    }
}
