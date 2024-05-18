use anyhow::{Context, Result};
use entity::{
    document,
    document_version::{ActiveModel, Column, Entity, Model},
    embedding,
};
use migration::{
    sea_orm::{
        self, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, FromQueryResult,
        QueryFilter, QuerySelect, RelationTrait, Set,
    },
    JoinType,
};

pub struct DocumentVersionRepo<'a>(&'a DatabaseConnection);

#[derive(Debug, Clone, FromQueryResult)]
pub struct DocumentVersionWithIsEmbedded {
    pub id: i32,
    pub project_version_project_id: i32,
    pub project_version_version: i32,
    pub document_id: i32,
    pub is_embedded: bool,
}

impl<'a> DocumentVersionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_pks_with_related_documents(
        &self,
        id: i32,
        project_id: i32,
        version: i32,
    ) -> Result<Option<(DocumentVersionWithIsEmbedded, document::Model)>> {
        let query = Entity::find_by_id((id, project_id, version))
            .column_as(embedding::Column::Id.count().ne(0), "is_embedded")
            .find_also_related(document::Entity)
            .join(JoinType::LeftJoin, document::Relation::Embedding.def())
            .filter(Column::Id.eq(id))
            .group_by(Column::Id)
            .group_by(Column::ProjectVersionProjectId)
            .group_by(Column::ProjectVersionVersion)
            .group_by(document::Column::Id);

        let result = query
            .into_model::<DocumentVersionWithIsEmbedded, document::Model>()
            .one(self.0)
            .await
            .context("Failed to get document version")?;

        let Some((result, Some(document))) = result else {
            return Ok(None);
        };

        Ok(Some((result, document)))
    }

    pub async fn find_by_pks(
        &self,
        id: i32,
        project_id: i32,
        version: i32,
    ) -> Result<Option<Model>> {
        let result = Entity::find_by_id((id, project_id, version))
            .one(self.0)
            .await
            .context("Failed to get document version")?;

        Ok(result)
    }

    pub async fn find_by_doc_id(&self, doc_id: i32) -> Result<Vec<Model>> {
        let result = Entity::find()
            .filter(Column::DocumentId.eq(doc_id))
            .all(self.0)
            .await
            .context("Failed to get document versions")?;

        Ok(result)
    }

    pub async fn create(
        &self,
        project_version_project_id: i32,
        project_version_version: i32,
        document_id: i32,
        tnx: Option<&DatabaseTransaction>,
    ) -> Result<(i32, i32, i32)> {
        let model = ActiveModel {
            project_version_project_id: Set(project_version_project_id),
            project_version_version: Set(project_version_version),
            document_id: Set(document_id),
            ..Default::default()
        };

        let res = match tnx {
            Some(tnx) => Entity::insert(model)
                .exec(tnx)
                .await
                .context("Failed to create document version")?,
            None => Entity::insert(model)
                .exec(self.0)
                .await
                .context("Failed to create document version")?,
        };

        Ok(res.last_insert_id)
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
                ..Default::default()
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
            .filter(Column::Id.is_not_in(exclude_documents))
            .all(tnx)
            .await
            .context("Failed to get document versions")?;

        // Bump the project version for each document version
        let models: Vec<ActiveModel> = result
            .iter_mut()
            .map(|model| {
                let project_version_version = model.project_version_version + 1;
                ActiveModel {
                    id: Set(model.id),
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

    pub async fn delete(&self, id: i32, project_id: i32, version: i32) -> Result<()> {
        let _ = Entity::delete_by_id((id, project_id, version))
            .exec(self.0)
            .await?;

        Ok(())
    }
}
