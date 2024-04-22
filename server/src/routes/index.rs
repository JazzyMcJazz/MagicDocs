use actix_web::{web, HttpResponse};
use tera::Context;

use crate::{database::Repo, server::AppState};

// GET /
pub async fn index(data: web::Data<AppState>) -> HttpResponse {
    let tera = &data.tera;
    let db = &data.conn;

    let projects = match db.projects().get_all().await {
        Ok(projects) => projects,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    dbg!(&projects);

    let mut context = Context::new();
    context.insert("projects", &projects);

    let Ok(html) = tera.render("index.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
