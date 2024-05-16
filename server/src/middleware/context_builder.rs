use axum::{
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
};

use tera::Context;

use crate::{
    database::Repo,
    models::Slugs,
    responses::HttpResponse,
    server::AppState,
    utils::{config::Config, context_data::UserData, extractor::Extractor},
};

pub async fn context_builder(
    State(app_data): State<AppState>,
    Path(path): Path<Slugs>,
    mut req: Request,
    next: Next,
) -> Response {
    let Ok(claims) = Extractor::claims(&req) else {
        return HttpResponse::InternalServerError().finish();
    };

    let config = Config::default();
    let user_data = UserData::from_claims(&claims);
    let env = config.rust_env();

    let db = app_data.conn.to_owned();
    let projects = match db.projects().all(&user_data).await {
        Ok(projects) => projects,
        Err(_) => Vec::new(),
    };

    let active_project = Extractor::active_project(path.project_id(), &projects);

    let project_versions = match &active_project {
        Some(project) => match db.projects_versions().all(project.id).await {
            Ok(versions) => Some(versions),
            Err(e) => {
                tracing::error!("Failed to get project versions: {:?}", e);
                Some(vec![])
            }
        },
        None => None,
    };

    let documents = match &active_project {
        Some(project) => match path.version() {
            Some(version) => match db
                .documents()
                .all_only_id_and_name(project.id, version)
                .await
            {
                Ok(documents) => Some(documents),
                Err(e) => {
                    tracing::error!("Failed to get documents: {:?}", e);
                    Some(vec![])
                }
            },
            None => None,
        },
        None => None,
    };

    let active_document = match &documents {
        Some(documents) => Extractor::active_document(path.doc_id(), documents),
        None => None,
    };

    let mut context = Context::new();
    context.insert("path", req.uri().path());
    context.insert("project_version", &path.version());
    context.insert("user", &user_data);
    context.insert("env", &env);
    context.insert("projects", &projects);
    context.insert("project", &active_project);
    context.insert("project_versions", &project_versions);
    context.insert("documents", &documents);
    context.insert("document", &active_document);

    let is_finalized = Extractor::project_version_finalized(&documents);
    if let Some(is_finalized) = is_finalized {
        context.insert("is_finalized", &is_finalized);
    }

    if let Some(project_versions) = project_versions {
        let latest_version = project_versions.iter().map(|m| m.version).max();

        context.insert("is_latest_version", &(latest_version == path.version()));
    }

    req.extensions_mut().insert(context);
    req.extensions_mut().insert(user_data);

    next.run(req).await
}
