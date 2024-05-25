use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Slugs {
    project_id: Option<i32>,
    version: Option<i32>,
    doc_id: Option<i32>,
    user_id: Option<String>,
    role_name: Option<String>,
}

impl Slugs {
    pub fn project_id(&self) -> Option<i32> {
        self.project_id
    }

    pub fn version(&self) -> Option<i32> {
        self.version
    }

    pub fn doc_id(&self) -> Option<i32> {
        self.doc_id
    }

    pub fn project_all(&self) -> (Option<i32>, Option<i32>, Option<i32>) {
        (self.project_id, self.version, self.doc_id)
    }

    pub fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }

    pub fn role_name(&self) -> Option<String> {
        self.role_name.clone()
    }
}
