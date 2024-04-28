mod document_repo;
mod project_repo;

use document_repo::DocumentRepo;
use project_repo::ProjectRepo;

use migration::sea_orm::DatabaseConnection;

pub trait Repo {
    fn projects(&self) -> ProjectRepo;
    fn documents(&self) -> DocumentRepo;
}

impl Repo for DatabaseConnection {
    fn projects(&self) -> ProjectRepo {
        ProjectRepo::new(self)
    }
    fn documents(&self) -> DocumentRepo {
        DocumentRepo::new(self)
    }
}
