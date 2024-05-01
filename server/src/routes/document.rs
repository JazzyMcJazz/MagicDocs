use actix_web::{
    web::{Data, Form, Path},
    HttpRequest, HttpResponse,
};
use serde::Deserialize;

use crate::{
    database::Repo,
    models::{CreateDocumentForm, StartCrawlerForm},
    parsing::Markdown,
    server::AppState,
    utils::{extractor::Extractor, traits::Htmx},
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
    req: HttpRequest,
) -> HttpResponse {
    let _db = &data.conn;
    let path = info.into_inner();
    let form = form.into_inner();

    dbg!(&path, &form);

    let (status, header) = req.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((header, format!("/projects/{}/documents/crawler", path.id)))
        .finish()
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
