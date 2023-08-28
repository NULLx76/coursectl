use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::{
    BrightspaceStudent, BrightspaceStudentList, GitlabApiResponse, ProjectInfo, Student,
};
use color_eyre::{
    eyre::{eyre, Context, Result},
    Help,
};
use gitlab::{
    api::{projects, Client, FormParams, Query, RestClient},
    Gitlab, ProjectId,
};
use http::{header, request::Builder as RequestBuilder};
use itertools::Itertools;

pub fn create_individual_repos(
    client: &Gitlab,
    repo_name_prefix: &str,
    parent_namespace_id: u64,
    student_list: impl AsRef<Path>,
    template_url: &str,
) -> Result<()> {
    let file = fs::read_to_string(student_list).wrap_err("failed to read student list file")?;
    let student_list: BrightspaceStudentList =
        serde_json::from_str(&file).wrap_err("error reading student json file")?;
    let students: Result<Vec<Student>> = student_list
        .students
        .into_iter()
        .map(BrightspaceStudent::try_into)
        .collect();

    let parent_project_names: Vec<String> =
        crate::projects::get_projects_by_group(client, parent_namespace_id)
            .wrap_err("failed getting projects under give parent id")?
            .into_iter()
            .map(|p| p.name)
            .collect();

    for s in students.wrap_err("failed to convert brightspace students into students")? {
        let name = format!("{repo_name_prefix} - {}", s.netid);

        if parent_project_names.iter().any(|pn| pn == &name) {
            println!("Skipping {}, already has a repo.", s.netid);
            continue;
        }

        create_repo_from_template(client, &[s], parent_namespace_id, &name, template_url)
            .wrap_err("failed creating repo")?;
    }

    Ok(())
}

pub fn create_repo_from_template(
    client: &Gitlab,
    students: &[Student],
    parent_namespace_id: u64,
    name: &str,
    template_url: &str,
) -> Result<()> {
    let endpoint = projects::CreateProject::builder()
        .visibility(gitlab::api::common::VisibilityLevel::Private)
        .import_url(template_url)
        .namespace_id(parent_namespace_id)
        .name(name)
        .emails_disabled(true)
        .build()
        .wrap_err("createproject builder")?;

    let project: ProjectInfo = endpoint.query(client).wrap_err("create project")?;

    invite(client, project.id, students).wrap_err("inviting students")?;

    Ok(())
}

/// invites users to a gitlab project by project id and student e-mail
/// See <https://docs.gitlab.com/ee/api/invitations.html>
pub fn invite(client: &Gitlab, id: ProjectId, students: &[Student]) -> Result<()> {
    let emails: String =
        Itertools::intersperse(students.iter().map(|s| s.email.as_str()), ",").collect();

    let endpoint = client
        .rest_endpoint(&format!("/api/v4/projects/{id}/invitations"))
        .wrap_err("getting endpoint url")?;

    let mut params = FormParams::default();
    params.push("email", emails);
    params.push("access_level", 30);
    let (mime, data) = params.into_body()?.unwrap();

    let req = RequestBuilder::new()
        .method("POST")
        .uri(endpoint.as_ref())
        .header(header::CONTENT_TYPE, mime);

    let rsp = client.rest(req, data).wrap_err("inviting users")?;

    let status = rsp.status();

    let v: GitlabApiResponse = if let Ok(v) = serde_json::from_slice(rsp.body()) {
        v
    } else {
        return Err(
            eyre!("invite server error {status}").with_section(|| format!("{:?}", rsp.body()))
        );
    };

    if !status.is_success() || v.status != "success" {
        return Err(eyre!("gitlab invite error").with_section(|| format!("{:?}", v.message)));
    }

    Ok(())
}
