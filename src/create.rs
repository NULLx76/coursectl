use crate::models::{GitlabApiResponse, ProjectInfo, Student};
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
