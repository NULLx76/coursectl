use std::collections::HashMap;

use color_eyre::{eyre::ContextCompat, Report, Result};
use gitlab::ProjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Student {
    pub netid: String,
    pub student_number: Option<u64>, // Employees don't have a student nr.
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StudentGroupEntry {
    pub group_name: String,
    pub full_name: String,
    pub netid: String,
    pub student_number: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub members: Vec<Student>,
}

impl Group {
    pub fn from_hm(hm: HashMap<String, Vec<Student>>) -> Vec<Self> {
        let mut output = Vec::with_capacity(hm.capacity());
        for (group, students) in hm {
            output.push(Group {
                name: group,
                members: students,
            });
        }

        output
    }
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

/// See: <https://docs.valence.desire2learn.com/res/apiprop.html#Version.ProductVersions>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BrightspaceProductVersions {
    pub product_code: String,
    pub latest_version: String,
    pub supported_versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BrightspaceGroupRecord {
    pub group_name: String,
    pub group_category: String,
    pub org_defined_id: Option<u64>,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl TryInto<Student> for BrightspaceGroupRecord {
    type Error = Report;

    fn try_into(self) -> std::result::Result<Student, Self::Error> {
        Ok(Student {
            netid: self
                .username
                .strip_suffix("@tudelft.nl")
                .wrap_err("failed to strip error")?
                .to_string(),
            student_number: self.org_defined_id,
            email: self.email,
        })
    }
}
