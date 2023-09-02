use crate::models::{BrightspaceClassList, BrightspaceClassListEntry, Student};
use color_eyre::{eyre::Context, Result};

#[inline]
fn classlist_url(ou: u64) -> String {
    format!("https://brightspace.tudelft.nl/d2l/api/le/1.72/{ou}/classlist/")
}

pub fn get_classlist(cookie: &str, ou: u64) -> Result<BrightspaceClassList> {
    let url = classlist_url(ou);
    let res: BrightspaceClassList = ureq::get(&url).set("Cookie", cookie).call()?.into_json()?;

    Ok(res)
}

pub fn get_students(cookie: &str, ou: u64) -> Result<Vec<Student>> {
    let classlist =
        get_classlist(cookie, ou).wrap_err("failed getting classlist from brightspace")?;

    classlist
        .into_iter()
        .filter(|e| e.role_id == Some(110))
        .map(BrightspaceClassListEntry::try_into)
        .collect()
}

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
