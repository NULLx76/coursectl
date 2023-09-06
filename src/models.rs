use std::collections::HashMap;

use color_eyre::{
    eyre::ContextCompat,
    Report, Result,
};
use gitlab::ProjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Student {
    pub netid: String,
    pub student_number: Option<u64>, // Employees don't have a student nr.
    pub email: String,
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

pub type BrightspaceClassList = Vec<BrightspaceClassListEntry>;

/// See [Brightspace Docs](https://docs.valence.desire2learn.com/res/enroll.html#Enrollment.ClasslistUser)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BrightspaceClassListEntry {
    pub identifier: String,
    pub profile_identifier: String,
    pub display_name: String,
    pub username: Option<String>,
    pub org_defined_id: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role_id: Option<u64>,
    pub last_accessed: Option<String>,
    pub is_online: bool,
    pub classlist_role_display_name: String,
}

impl TryInto<Student> for BrightspaceClassListEntry {
    type Error = Report;

    fn try_into(self) -> Result<Student> {
        Ok(Student {
            email: self.email.wrap_err("student missing email")?,
            student_number: self
                .org_defined_id
                .as_ref()
                .wrap_err("student missing student nr.")?
                .parse()
                .ok(),

            netid: self
                .username
                .wrap_err("student missing netid")?
                .strip_suffix("@tudelft.nl")
                .wrap_err("failed to strip @tudelft.nl from username")?
                .to_string(),
        })
    }
}
