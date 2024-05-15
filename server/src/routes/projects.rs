use std::{convert::Infallible, time::Duration};

use axum::{
    extract::{Path, Request, State},
    response::{sse::Event, Response, Sse},
    Form,
};
use futures_util::Stream;
use http::HeaderMap;

use crate::{
    database::Repo,
    langchain::{LLMProvider, Langchain},
    models::{CreateProjectForm, Slugs},
    responses::HttpResponse,
    server::AppState,
    utils::{
        extractor::Extractor,
        traits::{Htmx, TryRender},
    },
};

pub async fn new(State(data): State<AppState>, req: Request) -> Response {
    let context = Extractor::context(&req);
    data.tera.try_render("projects/new.html", &context)
}

// ListView: /projects
pub async fn create(
    State(data): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateProjectForm>,
) -> Response {
    let db = &data.conn;

    let Ok(id) = db.projects().create(form.name, form.description).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let (status, header) = headers.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((header, format!("/projects/{}", id)))
        .finish()
}

pub async fn redirect_to_latest(
    State(data): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Response {
    let db = &data.conn;

    let Ok(version) = db
        .projects_versions()
        .find_latest_version_number_or_create(id)
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let (status, header) = headers.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((header, format!("/projects/{}/v/{}", id, version)))
        .finish()
}

// DetailView: /projects/{id}
pub async fn detail(data: State<AppState>, req: Request) -> Response {
    let context = Extractor::context(&req);
    data.tera.try_render("projects/details.html", &context)
}

pub async fn finalize(
    data: State<AppState>,
    Path(path): Path<Slugs>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    let Some(project_id) = path.project_id() else {
        return Err(HttpResponse::BadRequest().finish());
    };
    let Some(version) = path.version() else {
        return Err(HttpResponse::BadRequest().finish());
    };

    let db = &data.conn;

    // TODO: Implement permission check
    // if has_permission {
    //     return Err(HttpResponse::Forbidden().finish());
    // }

    let mut documents = match db.documents().find_unembedded(project_id, version).await {
        Ok(documents) => documents,
        Err(e) => {
            tracing::error!("Failed to fetch documents: {:?}", e);
            return Err(HttpResponse::InternalServerError().finish());
        }
    };

    if documents.is_empty() {
        return Err(HttpResponse::NotFound().finish());
    }

    let stream = async_stream::stream! {
        let db = &data.conn;
        let lc = Langchain::new(LLMProvider::OpenAI);

        while let Some(document) = documents.pop() {
            yield Ok(Event::default().data(format!("Embedding Document:\n\"{}\"", document.name)));
            let embeddings = match lc.embed(&document.content).await {
                Ok(embeddings) => embeddings,
                Err(e) => {
                    tracing::error!("Failed to embed document: {:?}", e);
                    yield Ok(Event::default().data(format!("Failed to embed document: {:?}", e)));
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            match db.embeddings().create_many(document.id, embeddings).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to save embeddings: {:?}", e);
                    yield Ok(Event::default().data(format!("Failed to save embeddings: {:?}", e)));
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            };
        }

    };

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    );

    Ok(sse)
}
