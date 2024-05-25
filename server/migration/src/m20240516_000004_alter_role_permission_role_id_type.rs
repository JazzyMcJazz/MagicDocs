use sea_orm_migration::prelude::*;

use crate::m20240422_000001_create_tables::RolePermission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(RolePermission::Table)
                .modify_column(ColumnDef::new(RolePermission::RoleId).string().not_null())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(RolePermission::Table)
                .modify_column(ColumnDef::new(RolePermission::RoleId).integer().not_null())
                .to_owned()
        ).await
    }
}
