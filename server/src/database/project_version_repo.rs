use anyhow::{Context, Result};
use entity::project_version::{ActiveModel, Column, Entity, Model};
use migration::sea_orm::{prelude::*, EntityTrait, QueryFilter, QueryOrder, Set};

pub struct ProjectVersionRepo<'a>(&'a DatabaseConnection);

impl<'a> ProjectVersionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_latest_by_project_id(&self, id: i32) -> Result<Option<Model>> {
        let res = Entity::find()
            .filter(Column::ProjectId.eq(id))
            .order_by_asc(Column::CreatedAt)
            .one(self.0)
            .await?;

        Ok(res)
    }

    pub async fn create(&self, project_id: i32) -> Result<i32> {
        let model = ActiveModel {
            project_id: Set(project_id),
            ..Default::default()
        };

        let res = Entity::insert(model)
            .exec(self.0)
            .await
            .context("Failed to create project version")?;

        Ok(res.last_insert_id)
    }
}
