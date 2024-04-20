use actix_web::{
    cookie::{time::OffsetDateTime, Cookie},
    http::header::LOCATION,
    web, HttpRequest, HttpResponse,
};

use crate::server::AppState;

pub async fn logout(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let (cookie1, cookie2, cookie3) = expire_cookies();
    let response = HttpResponse::Found()
        .cookie(cookie1)
        .cookie(cookie2)
        .cookie(cookie3)
        .insert_header((LOCATION, "/"))
        .finish();

    let Some(refresh_token) = req.cookie("rf") else {
        return response;
    };

    match data.keycloak.logout(refresh_token.value()).await {
        Ok(_) => response,
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

fn expire_cookies() -> (Cookie<'static>, Cookie<'static>, Cookie<'static>) {
    let expires = OffsetDateTime::UNIX_EPOCH;

    let cookie1 = Cookie::build("id", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();
    let cookie2 = Cookie::build("ac", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();
    let cookie3 = Cookie::build("rf", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();

    (cookie1, cookie2, cookie3)
}
