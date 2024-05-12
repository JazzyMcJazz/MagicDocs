use anyhow::Result;
use entity::embedding::{ActiveModel, Entity};
use migration::sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::models::Embedding;

pub struct EmbeddingRepo<'a>(&'a DatabaseConnection);

impl<'a> EmbeddingRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
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
