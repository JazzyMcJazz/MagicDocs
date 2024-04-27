use actix_web::{web, HttpRequest, HttpResponse};

use crate::{server::AppState, utils::extractor::Extractor};

// GET /
pub async fn index(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let context = Extractor::extract_context(&req);

    let Ok(html) = tera.render("index.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
