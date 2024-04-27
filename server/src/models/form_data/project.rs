use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectForm {
    #[serde(rename = "project-name")]
    pub name: String,
    pub description: String,
}
