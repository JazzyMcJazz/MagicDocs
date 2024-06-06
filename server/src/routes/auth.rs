use axum::{
    extract::{Request, State},
    response::Response,
    Form,
};
use axum_extra::extract::CookieJar;
use http::header::LOCATION;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    keycloak::GrantType,
    responses::HttpResponse,
    server::AppState,
    utils::{cookies, extractor::Extractor},
};

#[derive(Clone, Deserialize)]
pub struct AuthForm {
    path: String,
}

pub async fn logout(
    data: State<AppState>,
    cookies: CookieJar,
    Form(form): Form<AuthForm>,
) -> Response {
    let expired_cookies = cookies::expire();

    let id_cookie = expired_cookies.0;
    let ac_cookie = expired_cookies.1;
    let rf_cookie = expired_cookies.2;

    let response = HttpResponse::build(StatusCode::FOUND)
        .cookie(id_cookie)
        .cookie(ac_cookie)
        .cookie(rf_cookie)
        .insert_header((LOCATION, form.path));

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

    let Ok(form) = Extractor::form_data::<AuthForm>(req).await else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok((id_cookie, access_cookie, refresh_cookie)) =
        cookies::from_token_response(token_response)
    else {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::build(StatusCode::FOUND)
        .cookie(id_cookie)
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .insert_header((LOCATION, form.path.to_owned()))
        .finish()
}
