use crate::models::BrightspaceStudentList;

const classlist_api: &str = "https://brightspace.tudelft.nl/d2l/api/le/1.72/{}/classlist/";

pub fn get_classlist(ou: u64) -> BrightspaceStudentList {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::models::BrightspaceStudentList;

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
        let _: BrightspaceStudentList = serde_json::from_str(data).unwrap();
    }
}
