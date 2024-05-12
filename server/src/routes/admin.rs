use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
};

use crate::{
    responses::HttpResponse,
    server::AppState,
    utils::{extractor::Extractor, traits::TryRender},
};

pub async fn dashboard(data: State<AppState>, req: Request) -> Response {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/dashboard.html", &context) else {
        return HttpResponse::InternalServerError()
            .body("Template error")
            .finish();
    };

    HttpResponse::Ok().body(html)
}

pub async fn users(data: State<AppState>, req: Request) -> Response {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/users.html", &context) else {
        return HttpResponse::InternalServerError()
            .body("Template error")
            .finish();
    };

    HttpResponse::Ok().body(html)
}

pub async fn roles(data: State<AppState>, req: Request) -> impl IntoResponse {
    let context = Extractor::context(&req);
    data.tera.try_render("admin/roles.html", &context)
}

pub async fn permissions(data: State<AppState>, req: Request) -> impl IntoResponse {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    tera.try_render("admin/permissions.html", &context)
}
