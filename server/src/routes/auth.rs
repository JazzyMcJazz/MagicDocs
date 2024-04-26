use actix_web::{web, HttpRequest, HttpResponse};

use crate::{
    keycloak::GrantType,
    server::AppState,
    utils::{cookies, traits::Htmx},
};

pub async fn logout(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let (cookie1, cookie2, cookie3) = cookies::expire();
    let (status, header) = req.redirect_status_and_header();

    dbg!(&status, &header);

    let response = HttpResponse::build(status)
        .cookie(cookie1)
        .cookie(cookie2)
        .cookie(cookie3)
        .insert_header((header, "/"))
        .finish();

    let Some(refresh_token) = req.cookie("rf") else {
        return response;
    };

    match data.keycloak.logout(refresh_token.value()).await {
        Ok(_) => response,
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn refresh(data: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let Some(refresh_token) = req.cookie("rf") else {
        return HttpResponse::BadRequest().finish();
    };

    let kc = data.keycloak.to_owned();

    let Ok(token_response) = kc
        .exchange_token(GrantType::RefreshToken(refresh_token.value()), req.path())
        .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok((id_cookie, access_cookie, refresh_cookie)) =
        cookies::from_token_response(&token_response)
    else {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok()
        .cookie(id_cookie)
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .insert_header(("HX-Refresh", "true"))
        .finish()
}
