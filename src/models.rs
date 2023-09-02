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

pub type BrightspaceStudentList = Vec<BrightspaceClassListEntry>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BrightspaceClassListEntry {
    pub identifier: String,
    pub profile_identifier: String,
    pub display_name: String,
    pub username: String,
    pub org_defined_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role_id: u64,
    pub last_accessed: Option<String>,
    pub is_online: bool,
    pub classlist_role_display_name: String,
}

impl TryInto<Student> for BrightspaceClassListEntry {
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
