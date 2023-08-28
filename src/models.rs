use std::collections::HashMap;

use color_eyre::{
    eyre::{Context, ContextCompat},
    Report, Result,
};
use gitlab::ProjectId;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Student {
    pub email: String,
    pub student_number: u64,
    pub netid: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectInfo {
    pub id: ProjectId,
    pub name: String,
    pub ssh_url_to_repo: String,
}

#[derive(Debug, Deserialize)]
pub struct GitlabApiResponse {
    pub status: String,
    #[serde(default)]
    pub message: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct BrightspaceStudentList {
    #[serde(alias = "Students")]
    pub students: Vec<BrightspaceStudent>,
}

#[derive(Debug, Deserialize)]
pub struct BrightspaceStudent {
    #[serde(alias = "Username")]
    pub username: String,
    #[serde(alias = "OrgDefinedID")]
    pub org_defined_id: String,
    pub email: String,
}

impl TryInto<Student> for BrightspaceStudent {
    type Error = Report;

    fn try_into(self) -> Result<Student> {
        Ok(Student {
            email: self.email,
            student_number: self
                .org_defined_id
                .parse()
                .wrap_err("failed to convert netid to number")?,

            netid: self
                .username
                .strip_suffix("@tudelft.nl")
                .wrap_err("failed to strip @tudelft.nl from username")?
                .to_string(),
        })
    }
}
