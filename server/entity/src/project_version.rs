//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "project_version")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub published: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::document_version::Entity")]
    DocumentVersion,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Project,
}

impl Related<super::document_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentVersion.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        super::document_version::Relation::Document.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::document_version::Relation::ProjectVersion
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
