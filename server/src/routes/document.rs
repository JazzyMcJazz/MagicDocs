use axum::{
    extract::{Path, Request, State},
    response::{sse::Event, Response, Sse},
    Form,
};
use futures_util::{pin_mut, Stream, StreamExt};
use http::HeaderMap;
use std::{convert::Infallible, time::Duration};
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
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    let Some(project_id) = path.project_id() else {
        return Err(HttpResponse::BadRequest().finish());
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
                    yield Ok(Event::default().data(msg));
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

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    ))
}

// DetailView: /projects/{id}/document/{doc_id}
pub async fn detail(data: State<AppState>, Path(path): Path<Slugs>, req: Request) -> Response {
    let mut context = Extractor::context(&req);
    let tera = &data.tera;
    let db = &data.conn;

    let Some(doc_id) = path.doc_id() else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok(document) = db.documents().find_by_id(doc_id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let mut document = match document {
        Some(doc) => doc,
        None => return HttpResponse::NotFound().finish(),
    };

    document.content = Markdown.to_html(&document.content);
    context.insert("document", &document);

    tera.try_render("projects/documents/details.html", &context)
}
