pub use sea_orm_migration::prelude::*;

pub mod m20240422_000001_create_tables;
pub mod m20240510_000002_create_embedding_table;
pub mod m20240516_000003_add_finalized_column;
pub mod m20240516_000004_alter_role_permission_role_id_type;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240422_000001_create_tables::Migration),
            Box::new(m20240510_000002_create_embedding_table::Migration),
            Box::new(m20240516_000003_add_finalized_column::Migration),
            Box::new(m20240516_000004_alter_role_permission_role_id_type::Migration),
        ]
    }
}
