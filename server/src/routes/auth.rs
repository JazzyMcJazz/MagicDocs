use axum::{
    extract::{Request, State},
    response::Response,
};
use axum_extra::extract::CookieJar;
use http::HeaderMap;
use reqwest::StatusCode;

use crate::{
    keycloak::GrantType,
    responses::HttpResponse,
    server::AppState,
    utils::{cookies, traits::Htmx},
};

pub async fn logout(data: State<AppState>, cookies: CookieJar, headers: HeaderMap) -> Response {
    let expired_cookies = cookies::expire();

    let id_cookie = expired_cookies.0;
    let ac_cookie = expired_cookies.1;
    let rf_cookie = expired_cookies.2;

    let (status, header) = headers.redirect_status_and_header();

    let response = HttpResponse::build(status)
        .cookie(id_cookie)
        .cookie(ac_cookie)
        .cookie(rf_cookie)
        .insert_header((header, "/".to_owned()));

    let Some(refresh_token) = cookies.get("rf") else {
        return response.finish();
    };

    match data.keycloak.logout(refresh_token.value()).await {
        Ok(_) => response.finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn refresh(data: State<AppState>, cookies: CookieJar, req: Request) -> Response {
    let Some(refresh_token) = cookies.get("rf") else {
        return HttpResponse::BadRequest().finish();
    };

    let kc = data.keycloak.to_owned();

    let Ok(token_response) = kc
        .exchange_token(
            GrantType::RefreshToken(refresh_token.value()),
            req.uri().path(),
        )
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok((id_cookie, access_cookie, refresh_cookie)) =
        cookies::from_token_response(token_response)
    else {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::build(StatusCode::OK)
        .cookie(id_cookie)
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .insert_header(("HX-Refresh", "true".to_owned()))
        .finish()
}
