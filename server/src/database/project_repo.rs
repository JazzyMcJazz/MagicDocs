use anyhow::{Context, Result};
use entity::{
    project::{ActiveModel, Column, Entity, Model},
    role_permission,
    sea_orm_active_enums::PermissionEnum,
    user_permission,
};
use migration::sea_orm::{
    entity::prelude::*, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Set,
};

use crate::utils::context_data::UserData;

pub struct ProjectRepo<'a>(&'a DatabaseConnection);

impl<'a> ProjectRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn all(&self, user: &UserData) -> Result<Vec<Model>> {
        if user.is_admin {
            Entity::find()
                .all(self.0)
                .await
                .context("Failed to get all projects")
        } else {
            let subquery_user_permission = user_permission::Entity::find()
                .filter(user_permission::Column::UserId.eq(user.id.clone()))
                .filter(user_permission::Column::Type.eq(PermissionEnum::Read))
                .select_only()
                .column(user_permission::Column::ProjectId)
                .distinct()
                .into_query();

            let subquery_role_permission = role_permission::Entity::find()
                .filter(role_permission::Column::RoleId.is_in(user.roles.clone()))
                .filter(role_permission::Column::Type.eq(PermissionEnum::Read))
                .select_only()
                .column(role_permission::Column::ProjectId)
                .distinct()
                .into_query();

            let query = Entity::find()
                .filter(
                    Expr::col(Column::Id)
                        .in_subquery(subquery_user_permission)
                        .or(Expr::col(Column::Id).in_subquery(subquery_role_permission)),
                )
                .distinct();

            let result = query.all(self.0).await?;

            Ok(result)
        }
    }

    pub async fn all_with_user_permissions(
        &self,
        user_id: &str,
    ) -> Result<Vec<(Model, Vec<user_permission::Model>)>> {
        let projects = Entity::find()
            .all(self.0)
            .await
            .context("Failed to get all projects")?;

        let user_permissions = projects
            .load_many(
                user_permission::Entity::find().filter(user_permission::Column::UserId.eq(user_id)),
                self.0,
            )
            .await?;

        let res = projects.into_iter().zip(user_permissions).collect();

        Ok(res)
    }

    pub async fn all_with_role_permissions(
        &self,
        role_id: &str,
    ) -> Result<Vec<(Model, Vec<role_permission::Model>)>> {
        let projects = Entity::find()
            .all(self.0)
            .await
            .context("Failed to get all projects")?;

        let role_permissions = projects
            .load_many(
                role_permission::Entity::find().filter(role_permission::Column::RoleId.eq(role_id)),
                self.0,
            )
            .await?;

        let res = projects.into_iter().zip(role_permissions).collect();

        Ok(res)
    }

    pub async fn create(&self, name: String, description: String) -> Result<i32> {
        let model = ActiveModel {
            name: Set(name),
            description: Set(description),
            ..Default::default()
        };

        let res = Entity::insert(model)
            .exec(self.0)
            .await
            .context("Failed to create project")?;

        Ok(res.last_insert_id)
    }
}
