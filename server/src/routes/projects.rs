use actix_web::{web, HttpRequest, HttpResponse};

use crate::{server::AppState, utils::extractor::Extractor};

pub async fn new(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let context = Extractor::extract_context(&req);
    let tera = &data.tera;

    let Ok(html) = tera.render("projects/new.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

// ListView: /projects
// pub async fn list(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
//     todo!();
// }

// DetailView: /projects/{id}
// pub async fn detail(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
//     todo!();
// }
