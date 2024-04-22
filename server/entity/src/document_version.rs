//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "document_version")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub project_version_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub document_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::document::Entity",
        from = "Column::DocumentId",
        to = "super::document::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Document,
    #[sea_orm(
        belongs_to = "super::project_version::Entity",
        from = "Column::ProjectVersionId",
        to = "super::project_version::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ProjectVersion,
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<super::project_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectVersion.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_version::Relation::Project.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::project_version::Relation::DocumentVersion
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}