use sea_orm_migration::{prelude::*, sea_orm::{EnumIter, Iterable}, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

const CURRENT_TIMESTAMP: sea_query::expr::SimpleExpr = SimpleExpr::Keyword(Keyword::CurrentTimestamp);

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        //=================//
        // PERMISSION ENUM //
        //=================//
        manager
            .create_type(
                Type::create()
                    .as_enum(PermissionEnum)
                    .values(Permission::iter())
                    .to_owned()
            )
            .await?;

        //===============//
        // PROJECT TABLE //
        //===============//
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::Name).string().not_null())
                    .col(ColumnDef::new(Project::Description).string().not_null())
                    .col(ColumnDef::new(Project::CreatedAt).timestamp().not_null().default(CURRENT_TIMESTAMP))
                    .to_owned(),
            )
            .await?;

        //========================//
        // USER PERMISSIONS TABLE //
        //========================//
        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserPermission::UserId).string().not_null())
                    .col(ColumnDef::new(UserPermission::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(UserPermission::Type)
                            .enumeration(PermissionEnum, Permission::iter())
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(UserPermission::UserId)
                            .col(UserPermission::ProjectId)
                            .col(UserPermission::Type)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_permissions_project_id")
                            .from(UserPermission::Table, UserPermission::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        //========================//
        // ROLE PERMISSIONS TABLE //
        //========================//
        manager
            .create_table(
                Table::create()
                    .table(RolePermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RolePermission::RoleId).integer().not_null())
                    .col(ColumnDef::new(RolePermission::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(RolePermission::Type)
                            .enumeration(PermissionEnum, Permission::iter()).not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(RolePermission::RoleId)
                            .col(RolePermission::ProjectId)
                            .col(RolePermission::Type)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_project_id")
                            .from(RolePermission::Table, RolePermission::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        //========================//
        // PROJECT VERSIONS TABLE //
        //========================//
        manager
            .create_table(
                Table::create()
                    .table(ProjectVersion::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ProjectVersion::Id).integer().primary_key().auto_increment().not_null())
                    .col(ColumnDef::new(ProjectVersion::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectVersion::Published).boolean().not_null().default(false))
                    .col(ColumnDef::new(ProjectVersion::CreatedAt).date_time().not_null().default(CURRENT_TIMESTAMP))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_versions_project_id")
                            .from(ProjectVersion::Table, ProjectVersion::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        //=================//
        // DOCUMENTS TABLE //
        //=================//
        manager
            .create_table(
                Table::create()
                    .table(Document::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Document::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Document::Name).string().not_null())
                    .col(ColumnDef::new(Document::Content).text().not_null())
                    .col(ColumnDef::new(Document::Source).string())
                    .col(ColumnDef::new(Document::CreatedAt).timestamp().default(CURRENT_TIMESTAMP))
                    .to_owned(),
            )
            .await?;

        //==========================//
        // DOCUMENT VERSIONS TABLE //
        //==========================//
        manager
            .create_table(
                Table::create()
                    .table(DocumentVersion::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DocumentVersion::ProjectVersionId).integer().not_null())
                    .col(ColumnDef::new(DocumentVersion::DocumentId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(DocumentVersion::ProjectVersionId)
                            .col(DocumentVersion::DocumentId)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_document_versions_project_version_id")
                            .from(DocumentVersion::Table, DocumentVersion::ProjectVersionId)
                            .to(ProjectVersion::Table, ProjectVersion::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_document_versions_document_id")
                            .from(DocumentVersion::Table, DocumentVersion::DocumentId)
                            .to(Document::Table, Document::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DocumentVersion::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Document::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectVersion::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UserPermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(PermissionEnum).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserPermission {
    Table,
    UserId,
    ProjectId,
    Type,
}

#[derive(DeriveIden)]
enum RolePermission {
    Table,
    RoleId,
    ProjectId,
    Type,
}

#[derive(DeriveIden)]
enum ProjectVersion {
    Table,
    Id,
    ProjectId,
    CreatedAt,
    Published,
}

#[derive(DeriveIden)]
enum DocumentVersion {
    Table,
    ProjectVersionId,
    DocumentId,
}

#[derive(DeriveIden)]
enum Document {
    Table,
    Id,
    Name,
    Source,
    Content,
    CreatedAt,
}

#[derive(DeriveIden)]
struct PermissionEnum;

#[derive(DeriveIden, EnumIter)]
enum Permission {
    Create,
    Read,
    Update,
    Delete,
}