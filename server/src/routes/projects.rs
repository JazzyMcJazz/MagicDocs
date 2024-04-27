use actix_web::{
    web::{Data, Form, Path},
    HttpRequest, HttpResponse,
};
use serde::Deserialize;

use crate::{
    database::Repo,
    models::CreateProjectForm,
    server::AppState,
    utils::{extractor::Extractor, traits::Htmx},
};

pub async fn new(data: Data<AppState>, req: HttpRequest) -> HttpResponse {
    let context = Extractor::extract_context(&req);
    let tera = &data.tera;

    let Ok(html) = tera.render("projects/new.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

// ListView: /projects
pub async fn list(
    data: Data<AppState>,
    form: Form<CreateProjectForm>,
    req: HttpRequest,
) -> HttpResponse {
    let db = &data.conn;
    let project_data = form.into_inner();

    let Ok(id) = db
        .projects()
        .create((project_data.name, project_data.description))
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let (status, header) = req.redirect_status_and_header();
    HttpResponse::build(status)
        .insert_header((header, format!("/projects/{id}")))
        .finish()
}

#[derive(Deserialize)]
pub struct Info(i32);

// DetailView: /projects/{id}
pub async fn detail(data: Data<AppState>, info: Path<Info>, req: HttpRequest) -> HttpResponse {
    let mut context = Extractor::extract_context(&req);
    let tera = &data.tera;
    let db = &data.conn;
    let id = info.into_inner().0;

    let Ok(res) = db.projects().find_by_id(id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let Some(project) = res else {
        return HttpResponse::NotFound().finish();
    };

    context.insert("project", &project);

    let Ok(html) = tera.render("projects/details.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
