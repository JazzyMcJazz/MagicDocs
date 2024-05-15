use anyhow::Result;
use entity::embedding::{ActiveModel, Column, Entity};
use migration::{
    m20240422_000001_create_tables::{Document, DocumentVersion, ProjectVersion},
    m20240510_000002_create_embedding_table::Embedding as EmbeddingTbl,
    sea_orm::{self, DatabaseConnection, EntityTrait, Set, Statement},
    Alias, Cond, ConnectionTrait, Expr, JoinType, PostgresQueryBuilder, SelectStatement,
};

use crate::models::{Embedding, SearchResult};

pub struct EmbeddingRepo<'a>(&'a DatabaseConnection);

impl<'a> EmbeddingRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn similarity_search(
        &self,
        project_id: i32,
        version: i32,
        query: Vec<f32>,
    ) -> Result<Vec<SearchResult>> {
        let (sql, values) = SelectStatement::new()
            .from(EmbeddingTbl::Table)
            .column(Column::Text)
            .join(
                JoinType::InnerJoin,
                Document::Table,
                Expr::col((EmbeddingTbl::Table, EmbeddingTbl::DocumentId))
                    .equals((Document::Table, Document::Id)),
            )
            .join(
                JoinType::InnerJoin,
                DocumentVersion::Table,
                Expr::col((Document::Table, Document::Id))
                    .equals((DocumentVersion::Table, DocumentVersion::DocumentId)),
            )
            .join(
                JoinType::InnerJoin,
                ProjectVersion::Table,
                Expr::col((
                    DocumentVersion::Table,
                    DocumentVersion::ProjectVersionProjectId,
                ))
                .equals((ProjectVersion::Table, ProjectVersion::ProjectId)),
            )
            .cond_where(
                Cond::all()
                    .add(
                        Expr::col((ProjectVersion::Table, ProjectVersion::ProjectId))
                            .eq(project_id),
                    )
                    .add(Expr::col((ProjectVersion::Table, ProjectVersion::Version)).eq(version)),
            )
            .expr(Expr::cust_with_expr(
                "1 - (\"embedding\" <=> $1::vector) AS score",
                query,
            ))
            .order_by(Alias::new("score"), sea_orm::Order::Desc)
            .build(PostgresQueryBuilder)
            .to_owned();

        let stmt = Statement::from_sql_and_values(self.0.get_database_backend(), sql, values);

        let result: Vec<SearchResult> = self
            .0
            .query_all(stmt)
            .await?
            .iter()
            .map(|row| SearchResult {
                text: row.try_get::<String>("", "text").unwrap_or_default(),
                score: row.try_get::<f64>("", "score").unwrap_or_default(),
            })
            .filter(|r| r.score >= 0.6) // TODO: Filter in database query
            .collect();

        dbg!(&result);

        Ok(result)
    }

    pub async fn create_many(&self, document_id: i32, embeddings: Vec<Embedding>) -> Result<()> {
        let models = embeddings
            .iter()
            .map(|e| ActiveModel {
                document_id: Set(document_id),
                text: Set(e.text().to_owned()),
                embedding: Set(e.vector().to_owned()),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        Entity::insert_many(models).exec(self.0).await?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use migration::sea_orm::{ConnectOptions, Database};

//     use crate::utils::config::Config;

//     use super::*;

//     #[tokio::test]
//     async fn similarity_search() {
//         // dotenvy::dotenv().ok();
//         // let config = Config::default();
//         // let db_url = config.database_url();
//         // Establish connection to the database
//         // let opt = ConnectOptions::new(db_url);
//         // let conn = Database::connect(opt)
//         //     .await
//         //     .expect("Failed to connect to the database");

//         // let repo = EmbeddingRepo::new(&conn);

//         // let _ = repo.similarity_search(15, 1, vector.clone()).await.unwrap();

//         // let (sql, _) = Query::select()
//         //     .from(EmbeddingTbl::Table)
//         //     .column(Column::Text)
//         //     .join(
//         //         JoinType::InnerJoin,
//         //         Document::Table,
//         //         Expr::col((EmbeddingTbl::Table, EmbeddingTbl::DocumentId)).equals((Document::Table, Document::Id))
//         //     )
//         //     .join(
//         //         JoinType::InnerJoin,
//         //         DocumentVersion::Table,
//         //         Expr::col((DocumentVersion::Table, DocumentVersion::DocumentId)).equals((Document::Table, Document::Id))
//         //     )
//         //     .join(
//         //         JoinType::InnerJoin,
//         //         ProjectVersion::Table,
//         //         Expr::col((ProjectVersion::Table, ProjectVersion::Version)).equals((DocumentVersion::Table, DocumentVersion::ProjectVersionVersion))
//         //     )
//         //     .cond_where(
//         //         Cond::all()
//         //             .add(Expr::col((ProjectVersion::Table, ProjectVersion::ProjectId)).eq(1))
//         //             .add(Expr::col((ProjectVersion::Table, ProjectVersion::Version)).eq(1))
//         //     )
//         //     .expr(Expr::cust_with_expr("embedding <=> $1 as similarity", vector))
//         //     .order_by(Alias::new("similarity"), sea_orm::Order::Desc)
//         //     .build(PostgresQueryBuilder)
//         //     .to_owned();

//         // println!("{:?}", &sql);
//         // assert_eq!(sql, "SELECT \"text\", embedding <=> $1 as similarity FROM \"embedding\" INNER JOIN \"document\" ON \"embedding\".\"document_id\" = \"document\".\"id\" INNER JOIN \"document_version\" ON \"document_version\".\"document_id\" = \"document\".\"id\" INNER JOIN \"project_version\" ON \"project_version\".\"version\" = \"document_version\".\"project_version_version\" WHERE \"project_version\".\"project_id\" = $2 AND \"project_version\".\"version\" = $3 ORDER BY \"similarity\" DESC");
//     }
// }
