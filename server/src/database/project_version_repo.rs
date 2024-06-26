use anyhow::{Context, Result};
use entity::project_version::{ActiveModel, Column, Entity, Model};
use migration::sea_orm::{
    self, prelude::*, DatabaseTransaction, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

pub struct ProjectVersionRepo<'a>(&'a DatabaseConnection);

impl<'a> ProjectVersionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_latest(&self, id: i32) -> Result<Option<Model>> {
        let res = Entity::find()
            .filter(Column::ProjectId.eq(id))
            .order_by_desc(Column::Version)
            .one(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_by_pks(&self, project_id: i32, version: i32) -> Result<Option<Model>> {
        let res = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::Version.eq(version))
            .one(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_latest_version_number(&self, project_id: i32) -> Result<Option<i32>> {
        #[derive(FromQueryResult)]
        struct Version {
            version: i32,
        }

        let res = Entity::find()
            .column(Column::Version)
            .filter(Column::ProjectId.eq(project_id))
            .order_by_desc(Column::Version)
            .into_model::<Version>()
            .one(self.0)
            .await?;

        Ok(res.map(|model| model.version))
    }

    pub async fn all(&self, project_id: i32) -> Result<Vec<Model>> {
        let res = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .order_by_desc(Column::Version)
            .all(self.0)
            .await?;

        Ok(res)
    }

    pub async fn find_latest_version_number_or_create(&self, project_id: i32) -> Result<i32> {
        match self.find_latest(project_id).await? {
            Some(version) => Ok(version.version),
            None => {
                let (_, version) = self.create(project_id, None).await?;
                Ok(version)
            }
        }
    }

    pub async fn create(
        &self,
        project_id: i32,
        tnx: Option<&DatabaseTransaction>,
    ) -> Result<(i32, i32)> {
        let version = match self.find_latest_version_number(project_id).await? {
            Some(version) => version + 1,
            None => 1,
        };

        let model = ActiveModel {
            project_id: Set(project_id),
            version: Set(version),
            ..Default::default()
        };

        let res = match tnx {
            Some(tnx) => Entity::insert(model)
                .exec(tnx)
                .await
                .context("Failed to create project version")?,
            None => Entity::insert(model)
                .exec(self.0)
                .await
                .context("Failed to create project version")?,
        };

        Ok(res.last_insert_id)
    }

    pub async fn finalize(&self, project_id: i32, version: i32) -> Result<Model> {
        let model = ActiveModel {
            project_id: Set(project_id),
            version: Set(version),
            finalized: Set(true),
        };

        Ok(model.update(self.0).await?)
    }
}
