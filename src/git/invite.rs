use crate::models::{GitlabApiResponse, Student};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::{Result, Section};
use gitlab::api::common::AccessLevel;
use gitlab::api::projects::members;
use gitlab::api::{ignore, users, Client, FormParams, Query, RestClient};
use gitlab::{Gitlab, ProjectId, UserId};
use http::header;
use http::request::Builder as RequestBuilder;
use itertools::Itertools;
use serde::Deserialize;

/// Adds students to an already created gitlab project
///
/// This will either
/// * Find if the student already has an existing gitlab account and invite them based on that
/// * If not, invite the student via e-mail
pub fn add_students_to_project(
    client: &Gitlab,
    project: ProjectId,
    students: &[&Student],
    access_level: AccessLevel,
) -> Result<()> {
    // These people will be invited by email
    let mut to_invite = vec![];
    // These people will be added based on git ID
    let mut to_add = vec![];

    for &student in students {
        if let Some(id) = query_user(client, student)? {
            to_add.push(id);
        } else {
            to_invite.push(student);
        }
    }

    invite_by_email(client, project, &to_invite, access_level)?;
    invite_by_userinfo(client, project, &to_add, access_level)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    id: UserId,
}

/// Queries Gitlab to see if a certain student already has a gitlab account, if so, return `UserInfo`.
fn query_user(client: &Gitlab, student: &Student) -> Result<Option<UserInfo>> {
    fn query_user_by_username(client: &Gitlab, student: &str) -> Result<Option<UserInfo>> {
        let endpoint = users::Users::builder()
            .username(student)
            .build()
            .wrap_err("users builder")?;

        let mut users: Vec<UserInfo> = endpoint.query(client).wrap_err("query git for username")?;

        Ok((!users.is_empty()).then(|| users.swap_remove(0)))
    }

    fn query_user_by_email(client: &Gitlab, student: &Student) -> Result<Option<UserInfo>> {
        let endpoint = users::Users::builder()
            .search(&student.email)
            .build()
            .wrap_err("users builder")?;

        let mut users: Vec<UserInfo> = endpoint.query(client).wrap_err("query git for email")?;

        Ok((!users.is_empty()).then(|| users.swap_remove(0)))
    }

    // let student1 = format!("{}1", student.netid); // How much of a hack is this?

    Ok(query_user_by_username(client, &student.netid)?.or(query_user_by_email(client, student)?))
}

/// Invites students to an existing project if userid is known
fn invite_by_userinfo(
    client: &Gitlab,
    id: ProjectId,
    students: &[UserInfo],
    access_level: AccessLevel,
) -> Result<()> {
    if students.is_empty() {
        return Ok(());
    }

    let endpoint = members::AddProjectMember::builder()
        .project(id.value())
        .users(students.iter().map(|u| u.id.value()))
        .access_level(access_level)
        .build()
        .wrap_err("invite users by id builder")?;

    ignore(endpoint)
        .query(client)
        .wrap_err("call endpoint invite users by id")?;

    Ok(())
}

/// invites students to a git project by project id and student e-mail
/// See <https://docs.gitlab.com/ee/api/invitations.html>
pub fn invite_by_email(
    client: &Gitlab,
    id: ProjectId,
    students: &[&Student],
    access_level: AccessLevel,
) -> Result<()> {
    if students.is_empty() {
        return Ok(());
    }

    let emails: String =
        Itertools::intersperse(students.iter().map(|s| s.email.as_str()), ",").collect();

    let endpoint = client
        .rest_endpoint(&format!("/api/v4/projects/{id}/invitations"))
        .wrap_err("getting endpoint url")?;

    let mut params = FormParams::default();
    params.push("email", emails);
    params.push::<_, u64>("access_level", access_level.as_u64());
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
            eyre!("invite_by_email server error {status}").section(format!("{:?}", rsp.body()))
        );
    };

    if !status.is_success() {
        return Err(eyre!("git invite_by_email error").section(format!("{:?}", v.message)));
    }

    if v.status != "success" {
        eprintln!("git invite_by_email error: {:?}", v.message);
    }

    Ok(())
}
