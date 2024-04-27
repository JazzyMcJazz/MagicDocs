mod project_repo;

use project_repo::ProjectRepo;

use migration::sea_orm::DatabaseConnection;

pub trait Repo {
    fn projects(&self) -> ProjectRepo;
}

impl Repo for DatabaseConnection {
    fn projects(&self) -> ProjectRepo {
        ProjectRepo::new(self)
    }
}
