use std::vec;

use anyhow::{bail, Result};
use entity::{
    document::{ActiveModel, Column, Entity, Model, Relation},
    document_version, embedding, project_version,
};
use migration::{
    sea_orm::{
        prelude::*, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect,
        Set, Statement, TransactionTrait,
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
    ///     "document_version"."id",
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
    ///         "document_version"."project_version_project_id" = 1
    ///     AND
    ///         "document_version"."project_version_version" = 4
    /// GROUP BY
    ///     "document_version"."id",
    ///     "document"."name",
    ///     "project_version"."finalized"
    pub async fn all_only_id_and_name(
        &self,
        project_id: i32,
        project_version: i32,
    ) -> Result<Vec<DocumentWithoutContent>> {
        let res = Entity::find()
            .select_only()
            .column_as(document_version::Column::Id, "id")
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
            .filter(document_version::Column::ProjectVersionProjectId.eq(project_id))
            .filter(document_version::Column::ProjectVersionVersion.eq(project_version))
            .group_by(document_version::Column::Id)
            .group_by(Column::Name)
            .group_by(project_version::Column::Finalized)
            .into_model::<DocumentWithoutContent>()
            .all(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_by_id_and_version(
        &self,
        project_id: i32,
        project_version: i32,
        document_id: i32,
    ) -> Result<Option<Model>> {
        let res = Entity::find()
            .column(document_version::Column::Id)
            .join(JoinType::InnerJoin, Relation::DocumentVersion.def())
            .filter(document_version::Column::Id.eq(document_id))
            .filter(document_version::Column::ProjectVersionProjectId.eq(project_id))
            .filter(document_version::Column::ProjectVersionVersion.eq(project_version))
            .one(self.0)
            .await?;
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

    pub async fn create(&self, project_id: i32, name: &str, content: &str) -> Result<(i32, i32)> {
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
            name: Set(name.to_owned()),
            content: Set(content.to_owned()),
            ..Default::default()
        };

        let res = Entity::insert(model).exec(&tnx).await?;

        let document_id = res.last_insert_id;

        let doc_version_id = match self
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
            Ok(pks) => {
                tnx.commit().await?;
                pks.0
            }
            Err(e) => {
                tnx.rollback().await?;
                return Err(e);
            }
        };

        Ok((doc_version_id, project_version_version))
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
                self.0.get_database_backend(),
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

    /// Update a document
    ///
    /// 1. If the project version is finalized, bump project version, create new document, and add the new document version
    /// 2. If the document is embedded, create new document and update the relevant row in the document_version table
    /// 3. If the document is not embedded, update the document
    pub async fn update(
        &self,
        id: i32,
        project_id: i32,
        version: i32,
        (name, content): (&str, &str),
    ) -> Result<Option<i32>> {
        let Some(project_version) = self
            .0
            .projects_versions()
            .find_by_pks(project_id, version)
            .await?
        else {
            bail!("Project version not found")
        };

        let Some((document_version, document)) = self
            .0
            .documents_versions()
            .find_by_pks_with_related_documents(id, project_id, version)
            .await?
        else {
            bail!("Document version not found")
        };

        let (document_version, document) = (document_version.to_owned(), document.to_owned());

        // 1.
        if project_version.finalized {
            let tnx = self.0.begin().await?;

            let (_, new_version) = self
                .0
                .projects_versions()
                .create(project_id, Some(&tnx))
                .await?;

            if new_version - 1 != version {
                tnx.rollback().await?;
                bail!("You can only update documents from the latest version of the project");
            }

            self.0
                .documents_versions()
                .bump_project_version(project_id, version, vec![], &tnx)
                .await?;

            let model = ActiveModel {
                name: Set(name.to_owned()),
                content: Set(content.to_owned()),
                source: Set(document.source),
                ..Default::default()
            };

            let res = Entity::insert(model).exec(&tnx).await?;
            let new_document_id = res.last_insert_id;

            let model = document_version::ActiveModel {
                id: Set(document_version.id),
                project_version_project_id: Set(project_id),
                project_version_version: Set(new_version),
                document_id: Set(new_document_id),
            };
            model.save(&tnx).await?;

            tnx.commit().await?;
            return Ok(Some(new_version));
        }

        // 2.
        if document_version.is_embedded {
            let tnx = self.0.begin().await?;

            let model = ActiveModel {
                name: Set(name.to_owned()),
                content: Set(content.to_owned()),
                source: Set(document.source.to_owned()),
                ..Default::default()
            };

            let new_document_id = Entity::insert(model).exec(&tnx).await?.last_insert_id;

            document_version::ActiveModel {
                id: Set(document_version.id),
                project_version_project_id: Set(document_version.project_version_project_id),
                project_version_version: Set(document_version.project_version_version),
                document_id: Set(new_document_id),
            }
            .save(&tnx)
            .await?;

            tnx.commit().await?;
            return Ok(None);
        }

        // 3.
        let mut model = document.into_active_model();
        model.name = Set(name.to_owned());
        model.content = Set(content.to_owned());
        model.save(self.0).await?;

        Ok(None)
    }

    /// Delete a document
    ///
    /// 1. If the project version is finalized, create a new project_version and add every other document_version to it
    /// 2. If the document has multiple versions, delete the relevant row from the document_version table
    /// 3. If the document has only one version, delete the document (will cascade delete the document_version row)
    pub async fn delete(
        &self,
        document_version: i32,
        project_id: i32,
        version: i32,
    ) -> Result<Option<i32>> {
        let Some(project_version) = self
            .0
            .projects_versions()
            .find_by_pks(project_id, version)
            .await?
        else {
            bail!("Project version not found")
        };

        // 1.Project version is finalized
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
                .bump_project_version(project_id, version, vec![document_version], &tnx)
                .await?;

            tnx.commit().await?;
            return Ok(Some(new_version));
        }

        let Some(doc_v) = self
            .0
            .documents_versions()
            .find_by_pks(document_version, project_id, version)
            .await?
        else {
            bail!("Document version not found")
        };

        // 2. Document has multiple versions
        let document_versions = self
            .0
            .documents_versions()
            .find_by_doc_id(doc_v.document_id)
            .await?;

        if document_versions.is_empty() {
            bail!("Document not found");
        } else if document_versions.len() > 1 {
            tracing::info!("Document has mutliple versions, deleting the latest document_version");
            self.0
                .documents_versions()
                .delete(document_version, project_id, version)
                .await?;
            return Ok(None);
        }

        // 3. Document has only one version
        let id = document_versions[0].document_id;
        tracing::info!("Document has only one version, deleting the document");
        let _ = Entity::delete_by_id(id).exec(self.0).await?;

        Ok(None)
    }
}
