use anyhow::Result;
use migration::sea_orm::DatabaseConnection;

pub struct DocumentRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_id(&self, _id: i32) -> Result<Option<()>> {
        let _ = self.0;
        Ok(Some(()))
    }
}
