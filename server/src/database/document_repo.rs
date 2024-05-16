use std::vec;

use anyhow::{bail, Result};
use entity::{
    document::{ActiveModel, Column, Entity, Model, Relation},
    document_version, embedding, project_version,
};
use migration::{
    sea_orm::{
        prelude::*, DatabaseConnection, DbBackend, EntityTrait, QueryFilter, QuerySelect, Set,
        Statement, TransactionTrait,
    },
    JoinType, PostgresQueryBuilder, Query,
};

use crate::{
    models::DocumentWithoutContent,
    parsing::{Done, HtmlParser},
};

use super::Repo;

pub struct DocumentRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    /// Get all documents without their content
    ///
    /// Built query:
    ///
    /// ```sql
    /// SELECT
    ///     "document"."id",
    ///     "document"."name",
    ///     COUNT("embedding"."id") <> 0 AS "is_embedded",
    ///     "project_version"."finalized" AS "is_finalized"
    /// FROM
    ///     "document"
    /// INNER JOIN
    ///     "document_version" ON "document"."id" = "document_version"."document_id"
    /// INNER JOIN
    ///         "project_version" ON "document_version"."project_version_version" = "project_version"."version"
    ///     AND
    ///         "document_version"."project_version_project_id" = "project_version"."project_id"
    /// LEFT JOIN
    ///     "embedding" ON "document"."id" = "embedding"."document_id"
    /// WHERE
    ///         "project_version"."project_id" = 15
    ///     AND
    ///         "project_version"."version" = 1
    /// GROUP BY "document"."id", "project_version"."finalized"
    pub async fn all_only_id_and_name(
        &self,
        project_id: i32,
        project_version: i32,
    ) -> Result<Vec<DocumentWithoutContent>> {
        let res = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .column_as(embedding::Column::Id.count().ne(0), "is_embedded")
            .column_as(project_version::Column::Finalized, "is_finalized")
            .join(JoinType::InnerJoin, Relation::DocumentVersion.def())
            .join(
                JoinType::InnerJoin,
                document_version::Entity::belongs_to(project_version::Entity)
                    .from((
                        document_version::Column::ProjectVersionVersion,
                        document_version::Column::ProjectVersionProjectId,
                    ))
                    .to((
                        project_version::Column::Version,
                        project_version::Column::ProjectId,
                    ))
                    .into(),
            )
            .join(JoinType::LeftJoin, Relation::Embedding.def())
            .filter(project_version::Column::ProjectId.eq(project_id))
            .filter(project_version::Column::Version.eq(project_version))
            .group_by(Column::Id)
            .group_by(project_version::Column::Finalized)
            .into_model::<DocumentWithoutContent>()
            .all(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<Model>> {
        let res = Entity::find().filter(Column::Id.eq(id)).one(self.0).await?;
        Ok(res)
    }

    pub async fn find_unembedded(
        &self,
        project_id: i32,
        project_version: i32,
    ) -> Result<Vec<Model>> {
        let res = Entity::find()
            .join(JoinType::InnerJoin, Relation::DocumentVersion.def())
            .join(
                JoinType::InnerJoin,
                document_version::Relation::ProjectVersion.def(),
            )
            .join(JoinType::LeftJoin, Relation::Embedding.def())
            .filter(project_version::Column::ProjectId.eq(project_id))
            .filter(project_version::Column::Version.eq(project_version))
            .group_by(Column::Id)
            .having(embedding::Column::Id.count().eq(0))
            .all(self.0)
            .await?;

        Ok(res)
    }

    pub async fn create(
        &self,
        project_id: i32,
        name: String,
        content: String,
    ) -> Result<(i32, i32)> {
        let project_version = self.0.projects_versions().find_latest(project_id).await?;

        let tnx = self.0.begin().await?;

        let (mut project_version_id, mut project_version_version, finalized) = match project_version
        {
            Some(version) => (version.project_id, version.version, version.finalized),
            None => {
                let result = self
                    .0
                    .projects_versions()
                    .create(project_id, Some(&tnx))
                    .await?;
                (result.0, result.1, false)
            }
        };

        if finalized {
            let old_version = project_version_version.to_owned();
            (project_version_id, project_version_version) = self
                .0
                .projects_versions()
                .create(project_id, Some(&tnx))
                .await?;
            self.0
                .documents_versions()
                .bump_project_version(project_id, old_version, vec![], &tnx)
                .await?;
        }

        let model = ActiveModel {
            name: Set(name),
            content: Set(content),
            ..Default::default()
        };

        let res = Entity::insert(model).exec(&tnx).await?;

        let document_id = res.last_insert_id;

        match self
            .0
            .documents_versions()
            .create(
                project_version_id,
                project_version_version,
                document_id,
                Some(&tnx),
            )
            .await
        {
            Ok(_) => tnx.commit().await?,
            Err(e) => {
                tnx.rollback().await?;
                return Err(e);
            }
        };

        Ok((res.last_insert_id, project_version_version))
    }

    pub async fn create_many_from_documents(
        &self,
        project_id: i32,
        data: Vec<HtmlParser<Done>>,
    ) -> Result<()> {
        let project_version = self.0.projects_versions().find_latest(project_id).await?;

        let tnx = self.0.begin().await?;

        let (project_version_project_id, project_version_version) = match project_version {
            Some(version) => (version.project_id, version.version),
            None => {
                self.0
                    .projects_versions()
                    .create(project_id, Some(&tnx))
                    .await?
            }
        };

        let mut builder = Query::insert();
        let mut builder =
            builder
                .into_table(Entity)
                .columns(vec![Column::Name, Column::Content, Column::Source]);

        for doc in data {
            builder = builder.values_panic(vec![
                doc.name().into(),
                doc.content().into(),
                doc.source().into(),
            ]);
        }

        let (sql, values) = builder
            .returning(Query::returning().columns([Column::Id]))
            .build(PostgresQueryBuilder);

        let ids: Vec<Result<i32, DbErr>> = tnx
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql,
                values,
            ))
            .await?
            .iter()
            .map(|row| row.try_get::<i32>("", "id"))
            .collect();

        let document_ids = ids.into_iter().collect::<Result<Vec<_>, _>>()?;

        self.0
            .documents_versions()
            .create_many(
                project_version_project_id,
                project_version_version,
                document_ids,
                Some(&tnx),
            )
            .await?;

        tnx.commit().await?;

        Ok(())
    }

    /// Delete a document
    ///
    /// 1. If the project version is finalized, create a new project_version and add every document_version to it
    /// 2. If the document has multiple versions, delete the relevant row from the document_version table
    /// 3. If the document has only one version, delete the document
    pub async fn delete(&self, project_id: i32, version: i32, doc_id: i32) -> Result<Option<i32>> {
        let Some(project_version) = self
            .0
            .projects_versions()
            .find_by_pks(project_id, version)
            .await?
        else {
            bail!("Project version not found")
        };

        if project_version.finalized {
            let tnx = self.0.begin().await?;

            let (_, new_version) = self
                .0
                .projects_versions()
                .create(project_id, Some(&tnx))
                .await?;
            if new_version - 1 != version {
                tnx.rollback().await?;
                bail!("You can only delete documents from the latest version of the project");
            }

            self.0
                .documents_versions()
                .bump_project_version(project_id, version, vec![doc_id], &tnx)
                .await?;

            tnx.commit().await?;
            return Ok(Some(new_version));
        }

        let document_versions = self.0.documents_versions().find_by_doc_id(doc_id).await?;
        if document_versions.is_empty() {
            bail!("Document not found");
        } else if document_versions.len() > 1 {
            self.0
                .documents_versions()
                .delete(project_id, version, doc_id)
                .await?;
            return Ok(None);
        }

        tracing::info!("Document has only one version, deleting the document");
        let _ = Entity::delete_by_id(doc_id).exec(self.0).await?;

        Ok(None)
    }
}
