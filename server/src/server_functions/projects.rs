use super::models::{ProjectData, ProjectDocument};
use leptos::{
    server,
    server_fn::codec::{StreamingText, TextStream},
    ServerFnError,
};

#[server]
pub async fn get_projects() -> Result<Vec<ProjectData>, ServerFnError> {
    use crate::{database::Repo, server::AppState, utils::claims::Claims};
    use axum::Extension;
    use leptos::use_context;
    use leptos_axum::extract;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let user: Extension<Claims> = extract().await?;

    let projects = if user.is_admin() {
        match db.projects().all().await {
            Ok(projects) => projects,
            Err(_) => Vec::new(),
        }
    } else {
        match db
            .projects()
            .all_with_permission(&user.sub(), user.roles())
            .await
        {
            Ok(projects) => projects,
            Err(_) => Vec::new(),
        }
    }
    .iter()
    .map(|p| ProjectData {
        id: p.id,
        name: p.name.to_owned(),
        description: p.description.to_owned(),
        version: -1,
        versions: Vec::new(),
        finalized: false,
    })
    .collect::<Vec<_>>();

    Ok(projects)
}

#[server]
pub async fn get_project_data(
    project_id: i32,
    version: Option<i32>,
) -> Result<(ProjectData, Vec<ProjectDocument>), ServerFnError> {
    use crate::{database::Repo, server::AppState};
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        tracing::error!("Failed to get app state");
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let project = match db.projects().find_by_id(project_id).await {
        Ok(Some(project)) => project,
        _ => {
            tracing::error!("Project not found");
            return Err(ServerFnError::ServerError("Project not found".to_string()));
        }
    };

    let project_version = match version {
        Some(version) => {
            db.projects_versions()
                .find_by_pks(project_id, version)
                .await
        }
        None => db.projects_versions().find_latest(project_id).await,
    };

    let Ok(Some(project_version)) = project_version else {
        tracing::error!("Project version not found");
        return Err(ServerFnError::ServerError(
            "Project version not found".to_string(),
        ));
    };

    let Ok(documents) = db
        .documents()
        .all_only_id_and_name(project_id, project_version.version)
        .await
    else {
        tracing::error!("Failed to get documents");
        return Err(ServerFnError::ServerError(
            "Failed to get documents".to_string(),
        ));
    };

    let Ok(versions) = db.projects_versions().all(project_id).await else {
        tracing::error!("Failed to get versions");
        return Err(ServerFnError::ServerError(
            "Failed to get versions".to_string(),
        ));
    };

    let project = ProjectData {
        id: project.id,
        name: project.name,
        description: project.description,
        version: project_version.version,
        versions: versions.iter().map(|v| v.version).collect(),
        finalized: project_version.finalized,
    };

    let documents = documents
        .iter()
        .map(|d| ProjectDocument {
            id: d.id,
            name: d.name.to_owned(),
            is_embedded: d.is_embedded,
        })
        .collect::<Vec<_>>();

    Ok((project, documents))
}

#[server]
pub async fn get_project_versions(project_id: i32) -> Result<Vec<i32>, ServerFnError> {
    use crate::{database::Repo, server::AppState};
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        tracing::error!("Failed to get app state");
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let Ok(versions) = db.projects_versions().all(project_id).await else {
        tracing::error!("Failed to get versions");
        return Err(ServerFnError::ServerError(
            "Failed to get versions".to_string(),
        ));
    };

    let versions = versions.iter().map(|v| v.version).collect();

    Ok(versions)
}

#[server(CreateProject)]
pub async fn create_project(
    project_name: String,
    description: String,
) -> Result<i32, ServerFnError> {
    use crate::{database::Repo, server::AppState, utils::claims::Claims};
    use axum::Extension;
    use leptos::use_context;
    use leptos_axum::extract;

    let user: Extension<Claims> = extract().await?;
    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    if !user.is_admin() {
        return Err(ServerFnError::ServerError("Unauthorized".to_string()));
    }

    let db = &state.conn;

    let Ok(id) = db.projects().create(project_name, description).await else {
        return Err(ServerFnError::ServerError(
            "Failed to create project".to_string(),
        ));
    };

    Ok(id)
}

#[server(output = StreamingText)]
pub async fn finalize_project_version(
    project_id: i32,
    version: i32,
) -> Result<TextStream, ServerFnError> {
    use crate::{
        database::Repo,
        langchain::{LLMProvider, Langchain},
        server::AppState,
    };
    use http::header::{HeaderName, HeaderValue};
    use leptos::{expect_context, use_context};
    use leptos_axum::ResponseOptions;
    use std::str::FromStr;

    tracing::info!("Finalizing project version: {} - {}", project_id, version);

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let response = expect_context::<ResponseOptions>();

    let db = state.conn;

    let mut documents = match db.documents().find_unembedded(project_id, version).await {
        Ok(documents) => documents,
        Err(e) => {
            tracing::error!("Failed to fetch documents: {:?}", e);
            return Err(ServerFnError::ServerError(
                "Failed to fetch documents".to_string(),
            ));
        }
    };

    if documents.is_empty() {
        return Err(ServerFnError::ServerError(
            "No documents to embed".to_string(),
        ));
    }

    let stream = async_stream::stream! {
        let lc = Langchain::new(LLMProvider::OpenAI);

        let mut had_error = false;
        while let Some(document) = documents.pop() {
            yield Ok::<_, ServerFnError>(format!("Embedding Document: {}", document.name));
            let embeddings = match lc.embed(&document.content).await {
                Ok(embeddings) => embeddings,
                Err(e) => {
                    tracing::error!("Failed to embed document: {:?}", e);
                    yield Ok::<_, ServerFnError>(format!("Failed to embed document: {:?}", e));
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    had_error = true;
                    break;
                }
            };

            match db.embeddings().create_many(document.id, embeddings).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to save embeddings: {:?}", e);
                    yield Ok::<_, ServerFnError>(format!("Failed to save embeddings: {:?}", e));
                    had_error = true;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            };
        }

        if !had_error {
            db.projects_versions().finalize(project_id, version).await.ok();
        }
    };

    if let Ok(key) = HeaderName::from_str("X-Accel-Buffering") {
        let value = HeaderValue::from_static("no");
        response.insert_header(key, value);
    }

    Ok(TextStream::new(stream))
}
