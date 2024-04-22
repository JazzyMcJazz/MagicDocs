mod project_repo;

use project_repo::ProjectRepo;

use migration::sea_orm::DatabaseConnection;

pub trait Repo {
    type Error;

    fn projects(&self) -> ProjectRepo;
}

impl Repo for DatabaseConnection {
    type Error = migration::sea_orm::error::DbErr;

    fn projects(&self) -> ProjectRepo {
        ProjectRepo::new(self)
    }
}
