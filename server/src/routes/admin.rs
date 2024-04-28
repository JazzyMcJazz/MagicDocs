use actix_web::{web, HttpRequest, HttpResponse};

use crate::{server::AppState, utils::extractor::Extractor};

pub async fn dashboard(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/dashboard.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

pub async fn users(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/users.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

pub async fn roles(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/roles.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}

pub async fn permissions(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/permissions.html", &context) else {
        return HttpResponse::InternalServerError().body("Template error");
    };

    HttpResponse::Ok().body(html)
}
