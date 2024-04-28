use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use serde::Deserialize;

use crate::{database::Repo, server::AppState, utils::extractor::Extractor};

#[derive(Deserialize)]
pub struct Info {
    id: i32,
    doc_id: i32,
}

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

// DetailView: /projects/{id}/document/{doc_id}
pub async fn detail(data: Data<AppState>, info: Path<Info>, req: HttpRequest) -> HttpResponse {
    let context = Extractor::context(&req);
    let tera = &data.tera;
    let db = &data.conn;
    let path = info.into_inner();
    dbg!(path.id, path.doc_id);

    let Ok(_) = db.documents().find_by_id(path.id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    // let Some(project) = res else {
    //     return HttpResponse::NotFound().finish();
    // };

    // context.insert("project", &project);

    let Ok(html) = tera.render("projects/documents/details.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
