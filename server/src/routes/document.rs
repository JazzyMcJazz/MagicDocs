use actix_web::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        Error,
    },
    web::{Bytes, Data, Form, Path},
    HttpRequest, HttpResponse,
};
use futures_util::{pin_mut, StreamExt};
use serde::Deserialize;

use crate::{
    database::Repo,
    models::{CreateDocumentForm, StartCrawlerForm},
    parsing::Markdown,
    server::AppState,
    utils::{extractor::Extractor, traits::Htmx},
    web_crawler::crawler::{Crawler, StreamOutput},
};

pub async fn new(data: Data<AppState>, req: HttpRequest) -> HttpResponse {
    let context = Extractor::context(&req);
    let tera = &data.tera;

    let file = if req.path().ends_with("/crawler") {
        "projects/documents/new/crawler.html"
    } else {
        "projects/documents/new/editor.html"
    };

    let Ok(html) = tera.render(file, &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

#[derive(Debug, Deserialize)]
pub struct ProjectPathInfo {
    id: i32,
}

pub async fn list(
    data: Data<AppState>,
    form: Form<CreateDocumentForm>,
    info: Path<ProjectPathInfo>,
    req: HttpRequest,
) -> HttpResponse {
    let db = &data.conn;
    let document_data = form.into_inner();
    let path = info.into_inner();

    let Ok(id) = db
        .documents()
        .create(path.id, document_data.name, document_data.content)
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let (status, header) = req.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((header, format!("/projects/{}/documents/{}", path.id, id)))
        .finish()
}

pub async fn crawler(
    data: Data<AppState>,
    form: Form<StartCrawlerForm>,
    info: Path<ProjectPathInfo>,
    _req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let _db = &data.conn;
    let _path = info.into_inner();
    let form = form.into_inner();

    let mut results = vec![];

    let response_body = async_stream::stream! {
        let mut crawler = Crawler::new(form.url, form.depth).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        while let Some(r) = stream.next().await {
            match r {
                StreamOutput::Message(msg) => {
                    println!("{}", msg);
                    yield Ok::<Bytes, Error>(Bytes::from(msg));
                }
                StreamOutput::Result(res) => {
                    results = res;
                }
            }
        };
        for res in results {
            let _ = res.path();
            let _ = res.title();
            let _ = res.html();
        }
    };

    let response = HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(response_body);

    Ok(response)
}

#[derive(Deserialize)]
pub struct DocumentPathInfo {
    doc_id: i32,
}

// DetailView: /projects/{id}/document/{doc_id}
pub async fn detail(
    data: Data<AppState>,
    info: Path<DocumentPathInfo>,
    req: HttpRequest,
) -> HttpResponse {
    let mut context = Extractor::context(&req);
    let tera = &data.tera;
    let db = &data.conn;
    let path = info.into_inner();

    let Ok(document) = db.documents().find_by_id(path.doc_id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let mut document = match document {
        Some(doc) => doc,
        None => return HttpResponse::NotFound().finish(),
    };

    document.content = Markdown.to_html(&document.content);
    context.insert("document", &document);

    let Ok(html) = tera.render("projects/documents/details.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
