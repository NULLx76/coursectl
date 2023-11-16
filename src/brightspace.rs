use std::collections::HashMap;

use crate::models::{
    BrightspaceClassList, BrightspaceClassListEntry, BrightspaceGroupRecord, Group, Student,
};
use color_eyre::{eyre::Context, Result};
use http::Uri;
use itertools::Itertools;

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

const GROUP_EXPORT_URL: &str = "https://group-impexp.lti.tudelft.nl/export/";

pub fn get_groups(sessionid: &str, category: &str) -> Result<Vec<Group>> {
    let res = ureq::post(GROUP_EXPORT_URL)
        .set("Cookie", &format!("sessionid={sessionid}"))
        .send_form(&[("resource_link_id", "2116724775"), ("categories", category)])?;

    let s = res
        .into_string()?
        .lines()
        .filter(|line| !line.starts_with(','))
        .join("\n");

    let mut reader = csv::Reader::from_reader(s.as_bytes());

    let mut hm: HashMap<String, Vec<Student>> = HashMap::new();

    for row in reader.deserialize() {
        let student: BrightspaceGroupRecord = row?;
        let group_name = student.group_name.replace(' ', "");

        let s: Student = student.try_into()?;

        hm.entry(group_name)
            .and_modify(|e| e.push(s.clone()))
            .or_insert_with(|| vec![s]);
    }

    Ok(Group::from_hm(hm))
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
