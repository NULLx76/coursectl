use std::collections::HashMap;

use gitlab::ProjectId;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Student {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectInfo {
    pub id: ProjectId,
}

#[derive(Debug, Deserialize)]
pub struct GitlabApiResponse{
    pub status: String,
    #[serde(default)]
    pub message: HashMap<String, String>
}
