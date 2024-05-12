pub use sea_orm_migration::prelude::*;

mod m20240422_000001_create_tables;
mod m20240510_000002_create_embedding_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240422_000001_create_tables::Migration),
            Box::new(m20240510_000002_create_embedding_table::Migration),
        ]
    }
}
