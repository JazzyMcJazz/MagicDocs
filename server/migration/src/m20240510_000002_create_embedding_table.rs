use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Embedding::Table)
                .if_not_exists()
                .col(ColumnDef::new(Embedding::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Embedding::DocumentId).integer().not_null())
                .col(ColumnDef::new(Embedding::Text).text().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_embedding_document_id")
                        .from(Embedding::Table, Embedding::DocumentId)
                        .to(Document::Table, Document::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .to_owned()

        ).await?;

        let db = manager.get_connection();
        db
            .execute_unprepared("ALTER TABLE embedding ADD COLUMN embedding vector(1536) NOT NULL;")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(
            Table::drop()
                .table(Embedding::Table)
                .if_exists()
                .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
pub enum Document {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum Embedding {
    Table,
    Id,
    DocumentId,
    _Embedding, // For readability
    Text,
}