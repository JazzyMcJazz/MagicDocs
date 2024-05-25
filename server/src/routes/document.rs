use axum::{
    extract::{Path, Request, State},
    response::{sse::Event, IntoResponse, Response, Sse},
    Form,
};
use futures_util::{pin_mut, StreamExt};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use std::convert::Infallible;
use tokio::time::sleep;

use crate::{
    database::Repo,
    models::{CreateDocumentForm, Slugs, StartCrawlerForm},
    parsing::{HtmlParser, Markdown},
    responses::HttpResponse,
    server::AppState,
    utils::{
        extractor::Extractor,
        traits::{Htmx, TryRender},
    },
    web_crawler::crawler::{Crawler, StreamOutput},
};

pub async fn new(data: State<AppState>, req: Request) -> Response {
    let Ok(permissions) = Extractor::permissions(&req) else {
        return HttpResponse::InternalServerError().finish();
    };

    if !permissions.write() {
        return HttpResponse::Forbidden().finish();
    }

    let context = Extractor::context(&req);
    let tera = &data.tera;

    let file = if req.uri().path().ends_with("/crawler") {
        "projects/documents/new/crawler.html"
    } else {
        "projects/documents/new/editor.html"
    };

    tera.try_render(file, &context)
}

pub async fn create(
    State(data): State<AppState>,
    Path(path): Path<Slugs>,
    headers: HeaderMap,
    Form(form): Form<CreateDocumentForm>,
) -> Response {
    let db = &data.conn;
    let Some(id) = path.project_id() else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok((document_id, project_version)) =
        db.documents().create(id, form.name, form.content).await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let (status, header) = headers.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((
            header,
            format!(
                "/projects/{}/v/{}/documents/{}",
                id, project_version, document_id
            ),
        ))
        .finish()
}

pub async fn crawler(
    State(data): State<AppState>,
    Path(path): Path<Slugs>,
    Form(form): Form<StartCrawlerForm>,
) -> Response {
    let Some(project_id) = path.project_id() else {
        return HttpResponse::BadRequest().finish();
    };

    let stream = async_stream::stream! {
        let db = &data.conn;

        let mut results = vec![];
        let Ok(mut crawler) = Crawler::new(form.url, form.depth) else {
            yield Ok(Event::default().data("Error"));
            return;
        };
        let stream = crawler.start().await;
        pin_mut!(stream);
        while let Some(r) = stream.next().await {
            match r {
                StreamOutput::Message(msg) => {
                    yield Ok::<_, Infallible>(Event::default().data(msg));
                }
                StreamOutput::Result(res) => {
                    results.push(res);
                }
            }
        };

        yield Ok(Event::default().data("Processings results..."));

        let mut documents = vec![];
        for res in results {
            let message = format!("Processing {}", res.title());
            yield Ok(Event::default().data(message));
            let parser = HtmlParser::new(&res.title(), &res.html(), res.url());
            match parser.parse() {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    let message = format!("Error: {}", e);
                    yield Ok(Event::default().data(message));
                }
            }
            sleep(std::time::Duration::from_millis(100)).await;
        }

        yield Ok(Event::default().data("Saving documents..."));

        let _ = db.documents().create_many_from_documents(project_id, documents).await;

        yield Ok(Event::default().data("Done"));
        sleep(std::time::Duration::from_secs(1)).await;
    };

    let sse = Sse::new(stream);

    let mut res = sse.into_response();
    res.headers_mut()
        .insert("X-Accel-Buffering", HeaderValue::from_static("no"));
    res
}

pub async fn detail(
    State(data): State<AppState>,
    Path(path): Path<Slugs>,
    req: Request,
) -> Response {
    let mut context = Extractor::context(&req);
    let tera = &data.tera;
    let db = &data.conn;

    let (Some(project_id), Some(version), Some(doc_id)) = path.project_all() else {
        return HttpResponse::BadRequest().finish();
    };

    match *req.method() {
        Method::GET => {
            let Ok(document) = db
                .documents()
                .find_by_version_id(doc_id, project_id, version)
                .await
            else {
                return HttpResponse::InternalServerError().finish();
            };

            let Some(mut document) = document else {
                return HttpResponse::NotFound().finish();
            };

            document.content = Markdown.to_html(&document.content);
            context.insert("document", &document);

            tera.try_render("projects/documents/details.html", &context)
        }
        Method::DELETE => {
            let result = match db.documents().delete(doc_id, project_id, version).await {
                Ok(res) => res,
                Err(e) => {
                    tracing::error!("Error deleting document: {:?}", e);
                    return HttpResponse::InternalServerError().finish();
                }
            };

            let version = match result {
                Some(new_version) => new_version,
                None => version,
            };

            let (status, header) = req.headers().redirect_status_and_header();
            HttpResponse::build(status)
                .insert_header((header, format!("/projects/{}/v/{}", project_id, version)))
                .finish()
        }
        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}

pub async fn patch(
    State(data): State<AppState>,
    Path(path): Path<Slugs>,
    Form(form): Form<CreateDocumentForm>,
) -> Response {
    let db = &data.conn;

    let (Some(project_id), Some(version), Some(doc_id)) = path.project_all() else {
        return HttpResponse::BadRequest().finish();
    };

    dbg!(&form);

    match db
        .documents()
        .update(doc_id, project_id, version, (&form.name, &form.content))
        .await
    {
        Ok(Some(version)) => HttpResponse::build(StatusCode::OK)
            .insert_header((
                HeaderName::from_static("hx-redirect"),
                format!(
                    "/projects/{}/v/{}/documents/{}",
                    project_id, version, doc_id
                ),
            ))
            .finish(),
        Ok(None) => HttpResponse::build(StatusCode::OK)
            .insert_header(("HX-Refresh", "true".to_owned()))
            .finish(),
        Err(e) => {
            tracing::error!("Error updating document: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn editor(data: State<AppState>, Path(path): Path<Slugs>, req: Request) -> Response {
    let mut context = Extractor::context(&req);
    let tera = &data.tera;
    let db = &data.conn;

    let (Some(project_id), Some(version), Some(doc_id)) = path.project_all() else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok(Some(latest_project_version)) = db.projects_versions().find_latest(project_id).await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    if latest_project_version.version != version {
        return HttpResponse::Forbidden().finish();
    }

    let Ok(document) = db
        .documents()
        .find_by_version_id(doc_id, project_id, version)
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let document = match document {
        Some(doc) => doc,
        None => return HttpResponse::NotFound().finish(),
    };

    context.insert("document", &document);

    match req.method() {
        &Method::GET => tera.try_render("snippets/editor.html", &context),
        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}
