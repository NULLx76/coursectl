use crate::models::{BrightspaceClassList, BrightspaceClassListEntry, Student};
use color_eyre::{eyre::Context, Result};
use http::Uri;

const BRIGHTSPACE_API_VERSION: &str = "1.72";

pub fn get_classlist(base_url: &Uri, cookie: &str, ou: u64) -> Result<BrightspaceClassList> {
    let url = format!("{base_url}d2l/api/le/{BRIGHTSPACE_API_VERSION}/{ou}/classlist/");
    let res: BrightspaceClassList = ureq::get(&url).set("Cookie", cookie).call()?.into_json()?;

    Ok(res)
}

pub fn get_students(base_url: &Uri, cookie: &str, ou: u64) -> Result<Vec<Student>> {
    let classlist = get_classlist(base_url, cookie, ou)
        .wrap_err("failed getting classlist from brightspace")?;

    classlist
        .into_iter()
        .filter(|e| e.role_id == Some(110))
        .map(BrightspaceClassListEntry::try_into)
        .collect()
}

/// <https://docs.valence.desire2learn.com/res/grade.html#get--d2l-api-le-(version)-(orgUnitId)-grades-(gradeObjectId)-values->
#[cfg(test)]
mod tests {
    use crate::models::BrightspaceClassList;

    #[test]
    pub fn try_parse() {
        let data = r#"
        [{
    "Identifier": "1",
    "ProfileIdentifier": "p1",
    "DisplayName": "Doe, John",
    "Username": "jdoe@tudelft.nl",
    "OrgDefinedId": "12345",
    "Email": "J.Doe@tudelft.nl",
    "FirstName": "John",
    "LastName": "Doe",
    "RoleId": 110,
    "LastAccessed": "2023-08-23T8:52:20.032Z",
    "IsOnline": false,
    "ClasslistRoleDisplayName": "Student"
  }]
        "#;
        let _: BrightspaceClassList = serde_json::from_str(data).unwrap();
    }
}
