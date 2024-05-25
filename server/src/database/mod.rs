mod document_repo;
mod document_version_repo;
mod embedding_repo;
mod permission_repo;
mod project_repo;
mod project_version_repo;

use document_repo::DocumentRepo;
use document_version_repo::DocumentVersionRepo;
use embedding_repo::EmbeddingRepo;
use permission_repo::{RolePermissionRepo, UserPermissionRepo};
use project_repo::ProjectRepo;
use project_version_repo::ProjectVersionRepo;

use migration::sea_orm::DatabaseConnection;

pub trait Repo {
    fn projects(&self) -> ProjectRepo;
    fn projects_versions(&self) -> ProjectVersionRepo;
    fn documents_versions(&self) -> DocumentVersionRepo;
    fn documents(&self) -> DocumentRepo;
    fn embeddings(&self) -> EmbeddingRepo;
    fn user_permissions(&self) -> UserPermissionRepo;
    fn role_permissions(&self) -> RolePermissionRepo;
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
    fn embeddings(&self) -> EmbeddingRepo {
        EmbeddingRepo::new(self)
    }
    fn user_permissions(&self) -> UserPermissionRepo {
        UserPermissionRepo::new(self)
    }
    fn role_permissions(&self) -> RolePermissionRepo {
        RolePermissionRepo::new(self)
    }
}
