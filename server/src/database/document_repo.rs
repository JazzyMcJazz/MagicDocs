use std::vec;

use anyhow::Result;
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
    models::DocumentWithIdAndName,
    parsing::{Done, HtmlParser},
};

use super::Repo;

pub struct DocumentRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn all_only_id_and_column(
        &self,
        project_id: i32,
        project_version: i32,
    ) -> Result<Vec<DocumentWithIdAndName>> {
        let res = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .column_as(embedding::Column::Id.count().ne(0), "is_embedded")
            .join(JoinType::InnerJoin, Relation::DocumentVersion.def())
            .join(
                JoinType::InnerJoin,
                document_version::Relation::ProjectVersion.def(),
            )
            .join(JoinType::LeftJoin, Relation::Embedding.def())
            .filter(project_version::Column::ProjectId.eq(project_id))
            .filter(project_version::Column::Version.eq(project_version))
            .group_by(Column::Id)
            .into_model::<DocumentWithIdAndName>()
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

        let (project_version_id, project_version_version) = match project_version {
            Some(version) => (version.project_id, version.version),
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

        let (project_version_project_id, project_version_version) = match project_version {
            Some(version) => (version.project_id, version.version),
            None => self.0.projects_versions().create(project_id).await?,
        };

        let tnx = self.0.begin().await?;

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
}
