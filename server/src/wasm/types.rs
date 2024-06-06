use leptos::{Params, Resource, RwSignal, ServerFnError};
use leptos_router::Params;

use crate::server_functions::models::{ProjectData, ProjectDocument};

pub type ProjectsResource = Resource<(), Result<Vec<ProjectData>, ServerFnError>>;
pub type ProjectDataContext = (ProjectData, Vec<ProjectDocument>);
pub type ProjectDataResource =
    Resource<(i32, Option<i32>), Result<(ProjectData, Vec<ProjectDocument>), ServerFnError>>;

#[derive(Params, PartialEq)]
pub struct ProjectParams {
    project_id: Option<i32>,
    document_id: Option<i32>,
}

impl ProjectParams {
    pub fn project_id(&self) -> Option<i32> {
        self.project_id
    }
    pub fn document_id(&self) -> Option<i32> {
        self.document_id
    }
}

#[derive(Params, PartialEq)]
pub struct AdminParams {
    user_id: Option<String>,
    role_name: Option<String>,
}

impl AdminParams {
    pub fn user_id(&self) -> Option<String> {
        self.user_id.to_owned()
    }
    pub fn role_name(&self) -> Option<String> {
        self.role_name.to_owned()
    }
}

#[derive(Params, PartialEq)]
pub struct VersionQuery {
    version: Option<i32>,
}

impl VersionQuery {
    pub fn version(&self) -> Option<i32> {
        self.version
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ChatUser {
    User(String),
    #[default]
    Assistant,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatMessage {
    pub user: ChatUser,
    pub key: u128,
    pub content: RwSignal<String>,
}
