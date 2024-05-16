use anyhow::{Context, Result};
use entity::document_version::{ActiveModel, Column, Entity, Model};
use migration::sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set,
};

pub struct DocumentVersionRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentVersionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_doc_id(&self, doc_id: i32) -> Result<Vec<Model>> {
        let result = Entity::find()
            .filter(Column::DocumentId.eq(doc_id))
            .all(self.0)
            .await
            .context("Failed to get document versions")?;

        Ok(result)
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

    pub async fn bump_project_version(
        &self,
        project_id: i32,
        version: i32,
        exclude_documents: Vec<i32>,
        tnx: &DatabaseTransaction,
    ) -> Result<()> {
        // Find all document versions for the project and version
        let mut result = Entity::find()
            .filter(Column::ProjectVersionProjectId.eq(project_id))
            .filter(Column::ProjectVersionVersion.eq(version))
            .filter(Column::DocumentId.is_not_in(exclude_documents))
            .all(tnx)
            .await
            .context("Failed to get document versions")?;

        // Bump the project version for each document version
        let models: Vec<ActiveModel> = result
            .iter_mut()
            .map(|model| {
                let project_version_version = model.project_version_version + 1;
                ActiveModel {
                    project_version_project_id: Set(project_id),
                    project_version_version: Set(project_version_version),
                    document_id: Set(model.document_id),
                }
            })
            .collect();

        // Insert the updated document versions
        Entity::insert_many(models)
            .exec(tnx)
            .await
            .context("Failed to bump project version for document versions")?;

        Ok(())
    }

    pub async fn delete(&self, project_id: i32, version: i32, doc_id: i32) -> Result<()> {
        let _ = Entity::delete_by_id((project_id, version, doc_id))
            .exec(self.0)
            .await?;

        Ok(())
    }
}
