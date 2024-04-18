use actix_web::{web, HttpResponse};
use tera::Context;

use crate::server::AppState;

// GET /
pub async fn index(data: web::Data<AppState>) -> HttpResponse {
    let tera = &data.tera;

    let context = Context::new();

    let Ok(html) = tera.render("index.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
