use anyhow::Result;
use entity::{
    document::{ActiveModel, Column, Entity, Model, Relation},
    document_version, project_version,
};
use migration::{
    sea_orm::{
        prelude::*, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
        TransactionTrait,
    },
    JoinType,
};

use crate::models::DocumentWithIdAndName;

use super::Repo;

pub struct DocumentRepo<'a>(&'a DatabaseConnection);

impl<'a> DocumentRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn all_only_id_and_column(
        &self,
        project_id: i32,
    ) -> Result<Vec<DocumentWithIdAndName>> {
        // let version_id = 1;

        let res = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::Name)
            .join(JoinType::InnerJoin, Relation::DocumentVersion.def())
            .join(
                JoinType::InnerJoin,
                document_version::Relation::ProjectVersion.def(),
            )
            .filter(project_version::Column::ProjectId.eq(project_id))
            // .filter(project_version::Column::Id.eq(version_id))
            .into_model::<DocumentWithIdAndName>()
            .all(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<Model>> {
        let res = Entity::find().filter(Column::Id.eq(id)).one(self.0).await?;

        Ok(res)
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
