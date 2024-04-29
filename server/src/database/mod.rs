mod document_repo;
mod document_version_repo;
mod project_repo;
mod project_version_repo;

use document_repo::DocumentRepo;
use document_version_repo::DocumentVersionRepo;
use project_repo::ProjectRepo;
use project_version_repo::ProjectVersionRepo;

use migration::sea_orm::DatabaseConnection;

pub trait Repo {
    fn projects(&self) -> ProjectRepo;
    fn projects_versions(&self) -> ProjectVersionRepo;
    fn documents_versions(&self) -> DocumentVersionRepo;
    fn documents(&self) -> DocumentRepo;
}

impl Repo for DatabaseConnection {
    fn projects(&self) -> ProjectRepo {
        ProjectRepo::new(self)
    }
    fn projects_versions(&self) -> ProjectVersionRepo {
        ProjectVersionRepo::new(self)
    }
    fn documents_versions(&self) -> DocumentVersionRepo {
        DocumentVersionRepo::new(self)
    }
    fn documents(&self) -> DocumentRepo {
        DocumentRepo::new(self)
    }
}
