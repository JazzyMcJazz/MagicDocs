use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Slugs {
    id: Option<i32>,
    version: Option<i32>,
    doc_id: Option<i32>,
}

impl Slugs {
    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn version(&self) -> Option<i32> {
        self.version
    }

    pub fn doc_id(&self) -> Option<i32> {
        self.doc_id
    }
}
