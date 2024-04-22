use entity::project;
use migration::{
    sea_orm::{DatabaseConnection, EntityTrait},
    DbErr,
};

pub struct ProjectRepo<'a>(&'a DatabaseConnection);

impl<'a> ProjectRepo<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self(db)
    }

    pub async fn get_all(&self) -> Result<Vec<project::Model>, DbErr> {
        project::Entity::find().all(self.0).await
    }
}
