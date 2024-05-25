use anyhow::Result;
use entity::{role_permission, sea_orm_active_enums::PermissionEnum, user_permission};
use migration::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait,
};

trait IntoString {
    fn to_string(&self) -> String;
}

impl IntoString for PermissionEnum {
    fn to_string(&self) -> String {
        match self {
            PermissionEnum::Create => "create".to_owned(),
            PermissionEnum::Read => "read".to_owned(),
            PermissionEnum::Update => "update".to_owned(),
            PermissionEnum::Delete => "delete".to_owned(),
        }
    }
}

pub struct UserPermissionRepo<'a>(&'a DatabaseConnection);

impl<'a> UserPermissionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_user_and_project_id(
        &self,
        user_id: &str,
        project_id: &i32,
    ) -> Result<Vec<user_permission::Model>> {
        let result = user_permission::Entity::find()
            .filter(user_permission::Column::UserId.eq(user_id))
            .filter(user_permission::Column::ProjectId.eq(*project_id))
            .all(self.0)
            .await?;

        Ok(result)
    }

    pub async fn create_many_for_user(
        &self,
        user_id: &str,
        permissions: Vec<(&i32, PermissionEnum)>,
    ) -> Result<()> {
        let models = permissions
            .iter()
            .map(|permission| user_permission::ActiveModel {
                user_id: Set(user_id.to_owned()),
                project_id: Set(permission.0.to_owned()),
                r#type: Set(permission.1.to_owned()),
            })
            .collect::<Vec<_>>();

        user_permission::Entity::insert_many(models)
            .exec(self.0)
            .await?;

        Ok(())
    }

    pub async fn delete_many_for_user(
        &self,
        user_id: &str,
        permissions: Vec<(&i32, PermissionEnum)>,
    ) -> Result<()> {
        let tnx = self.0.begin().await?;

        // Bad practice to do this in a loop,
        // but sea-orm does not support bulk delete
        // with composite primary keys.
        for permission in permissions {
            user_permission::Entity::delete_by_id((
                user_id.to_owned(),
                permission.0.to_owned(),
                permission.1.to_owned(),
            ))
            .exec(&tnx)
            .await?;
        }

        tnx.commit().await?;

        Ok(())
    }
}

pub struct RolePermissionRepo<'a>(&'a DatabaseConnection);

impl<'a> RolePermissionRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn find_by_project_id_in_roles(
        &self,
        role_ids: &Vec<String>,
        project_id: &i32,
    ) -> Result<Vec<role_permission::Model>> {
        let result = role_permission::Entity::find()
            .filter(role_permission::Column::RoleId.is_in(role_ids))
            .filter(role_permission::Column::ProjectId.eq(*project_id))
            .all(self.0)
            .await?;

        Ok(result)
    }

    pub async fn create_many_for_role(
        &self,
        role_name: &str,
        permissions: Vec<(&i32, PermissionEnum)>,
    ) -> Result<()> {
        let models = permissions
            .iter()
            .map(|permission| role_permission::ActiveModel {
                role_id: Set(role_name.to_owned()),
                project_id: Set(permission.0.to_owned()),
                r#type: Set(permission.1.to_owned()),
            })
            .collect::<Vec<_>>();

        role_permission::Entity::insert_many(models)
            .exec(self.0)
            .await?;

        Ok(())
    }

    pub async fn delete_many_for_role(
        &self,
        role_name: &str,
        permissions: Vec<(&i32, PermissionEnum)>,
    ) -> Result<()> {
        let tnx = self.0.begin().await?;

        // Bad practice to do this in a loop,
        // but sea-orm does not support bulk delete
        // with composite primary keys.
        for permission in permissions {
            role_permission::Entity::delete_by_id((
                role_name.to_owned(),
                permission.0.to_owned(),
                permission.1.to_owned(),
            ))
            .exec(&tnx)
            .await?;
        }

        tnx.commit().await?;

        Ok(())
    }
}
