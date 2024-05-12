use anyhow::{Context, Result};
use entity::document_version::{ActiveModel, Entity};
use migration::sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, Set};

pub struct DocumentVersionRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentVersionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    // pub async fn find_latest(&self, id: i32) -> Result<Option<i32>> {
    //     let result = Entity::find()
    //         .filter(Entity::Column::DocumentId.eq(id))
    //         .order_by(Entity::Column::Id.desc())
    //         .one(self.0)
    //         .await
    //         .context("Failed to get latest document version")?;

    //     Ok(result.map(|model| model.id))
    // }

    pub async fn create(
        &self,
        project_version_project_id: i32,
        project_version_version: i32,
        document_id: i32,
        tnx: Option<&DatabaseTransaction>,
    ) -> Result<()> {
        let model = ActiveModel {
            project_version_project_id: Set(project_version_project_id),
            project_version_version: Set(project_version_version),
            document_id: Set(document_id),
        };

        match tnx {
            Some(tnx) => Entity::insert(model)
                .exec(tnx)
                .await
                .context("Failed to create document version")?,
            None => Entity::insert(model)
                .exec(self.0)
                .await
                .context("Failed to create document version")?,
        };

        Ok(())
    }

    pub async fn create_many(
        &self,
        project_version_project_id: i32,
        project_version_version: i32,
        document_ids: Vec<i32>,
        tnx: Option<&DatabaseTransaction>,
    ) -> Result<()> {
        let models = document_ids
            .iter()
            .map(|&document_id| ActiveModel {
                project_version_project_id: Set(project_version_project_id),
                project_version_version: Set(project_version_version),
                document_id: Set(document_id),
            })
            .collect::<Vec<_>>();

        match tnx {
            Some(tnx) => Entity::insert_many(models)
                .exec(tnx)
                .await
                .context("Failed to create document versions")?,
            None => Entity::insert_many(models)
                .exec(self.0)
                .await
                .context("Failed to create document versions")?,
        };

        Ok(())
    }
}
