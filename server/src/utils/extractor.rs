use anyhow::{bail, Result};
use axum::extract::Request;
use entity::project;
use tera::Context;

use crate::models::DocumentWithoutContent;

use super::{claims::Claims, context_data::UserData};

pub struct Extractor;

impl Extractor {
    pub fn claims(req: &Request) -> Result<Claims> {
        let ext = req.extensions();
        match ext.get::<Claims>() {
            Some(claims) => Ok(claims.clone()),
            None => bail!("Claims not found in request"),
        }
    }

    pub fn _user_data(req: &Request) -> Result<UserData> {
        let ext = req.extensions();
        match ext.get::<UserData>() {
            Some(user_data) => Ok(user_data.clone()),
            None => bail!("User data not found in request"),
        }
    }

    pub fn context(req: &Request) -> Context {
        let ext = req.extensions();
        ext.get::<Context>().cloned().unwrap_or(Context::new())
    }

    pub fn active_project(
        project_id: Option<i32>,
        projects: &[project::Model],
    ) -> Option<project::Model> {
        match project_id {
            Some(project_id) => projects.iter().find(|p| p.id == project_id).cloned(),
            None => None,
        }
    }

    pub fn active_document(
        document_id: Option<i32>,
        documents: &[DocumentWithoutContent],
    ) -> Option<DocumentWithoutContent> {
        match document_id {
            Some(document_id) => documents.iter().find(|d| d.id == document_id).cloned(),
            None => None,
        }
    }

    pub fn project_version_finalized(
        documents: &Option<Vec<DocumentWithoutContent>>,
    ) -> Option<bool> {
        documents
            .as_ref()
            .map(|documents| documents.iter().all(|d| d.is_finalized))
    }
}
