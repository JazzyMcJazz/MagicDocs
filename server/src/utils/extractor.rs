use anyhow::{bail, Result};
use axum::{extract::Request, Form, Json, RequestExt};
use entity::project;
use tera::Context;

use crate::{middleware::Permissions, models::DocumentWithoutContent};

use super::{
    claims::{Claims, JwtTokens},
    context_data::UserData,
};

pub struct Extractor;

impl Extractor {
    pub fn claims(req: &Request) -> Result<Claims> {
        let ext = req.extensions();
        match ext.get::<Claims>() {
            Some(claims) => Ok(claims.clone()),
            None => bail!("Claims not found in request"),
        }
    }

    pub fn tokens(req: &Request) -> Result<JwtTokens> {
        let ext = req.extensions();
        match ext.get::<JwtTokens>() {
            Some(tokens) => Ok(tokens.clone()),
            None => bail!("Tokens not found in request"),
        }
    }

    pub fn permissions(req: &Request) -> Result<Permissions> {
        let ext = req.extensions();
        match ext.get::<Permissions>() {
            Some(permissions) => Ok(permissions.clone()),
            None => bail!("Permissions not found in request"),
        }
    }

    pub async fn form_data<T>(req: Request) -> Result<Form<T>>
    where
        T: serde::de::DeserializeOwned + Clone + 'static,
    {
        let res = req
            .extract::<Form<T>, _>()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(res)
    }

    pub async fn json<T>(req: Request) -> Result<T>
    where
        T: serde::de::DeserializeOwned + Clone + 'static,
    {
        let Json(res) = req
            .extract::<Json<T>, _>()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(res)
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
        match documents {
            Some(documents) => {
                if documents.is_empty() {
                    return None;
                }
            }
            None => (),
        };

        documents
            .as_ref()
            .map(|documents| documents.iter().all(|d| d.is_finalized))
    }
}
