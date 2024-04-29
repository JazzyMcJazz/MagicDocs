use anyhow::Result;
use entity::document::{ActiveModel, Entity};
use migration::sea_orm::{DatabaseConnection, EntityTrait, Set, TransactionTrait};

use super::Repo;

pub struct DocumentRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_id(&self, _id: i32) -> Result<Option<()>> {
        let _ = self.0;
        Ok(Some(()))
    }

    pub async fn create(&self, project_id: i32, name: String, content: String) -> Result<i32> {
        let project_version = self
            .0
            .projects_versions()
            .find_latest_by_project_id(project_id)
            .await?;

        let project_version_id = match project_version {
            Some(version) => version.id,
            None => self.0.projects_versions().create(project_id).await?,
        };

        let model = ActiveModel {
            name: Set(name),
            content: Set(content),
            ..Default::default()
        };

        let tnx = self.0.begin().await?;

        let res = Entity::insert(model).exec(&tnx).await?;

        let document_id = res.last_insert_id;

        match self
            .0
            .documents_versions()
            .create(project_version_id, document_id, Some(&tnx))
            .await
        {
            Ok(_) => tnx.commit().await?,
            Err(e) => {
                tnx.rollback().await?;
                return Err(e);
            }
        };

        Ok(res.last_insert_id)
    }
}
