use actix_web::{web, HttpRequest, HttpResponse};

use crate::{database::Repo, server::AppState, utils::extractor::Extractor};

// GET /
pub async fn index(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let db = &data.conn;
    let mut context = Extractor::extract_context(&req);

    let projects = match db.projects().get_all().await {
        Ok(projects) => projects,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    context.insert("projects", &projects);

    let Ok(html) = tera.render("index.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
