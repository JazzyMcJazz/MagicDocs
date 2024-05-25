use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use http::header::LOCATION;
use tera::Context;

use crate::{
    keycloak::GrantType,
    responses::HttpResponse,
    server::AppState,
    utils::{claims::JwtTokens, cookies, info::ConnectionInfo},
};

pub async fn authentication(
    State(app_data): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let info = ConnectionInfo::new(&req);

    let kc_code = info.query().and_then(|query| {
        url::form_urlencoded::parse(query.as_bytes())
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
    });

    // Check if the user is redirected from Keycloak
    if let Some(code) = kc_code {
        let tokens = match app_data
            .keycloak
            .exchange_token(GrantType::AuthorizationCode(code.clone()), &info.to_url())
            .await
        {
            Ok(token) => token,
            Err(e) => {
                tracing::error!("Error exchanging code: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let Ok(cookies) = cookies::from_token_response(tokens) else {
            tracing::error!("Error creating cookies");
            return HttpResponse::InternalServerError().finish();
        };

        return HttpResponse::build(StatusCode::FOUND)
            .cookie(cookies.0)
            .cookie(cookies.1)
            .cookie(cookies.2)
            .insert_header((LOCATION, info.path().to_owned()))
            .finish();
    }

    // Check if the user has a valid tokens or try to refresh the tokens
    let cookies = CookieJar::from_headers(req.headers());
    let (Some(id_token), Some(access_token)) = (cookies.get("id"), cookies.get("ac")) else {
        if let Some(refresh_token) = cookies.get("rf") {
            let refresh_token = refresh_token.clone();

            let tokens = match app_data
                .keycloak
                .exchange_token(
                    GrantType::RefreshToken(refresh_token.value()),
                    &info.to_url(),
                )
                .await
            {
                Ok(token) => token,
                Err(e) => {
                    tracing::error!("Error exchanging refresh token: {:?}", e);
                    let cookies = cookies::expire();
                    return HttpResponse::build(StatusCode::FOUND)
                        .cookie(cookies.0)
                        .cookie(cookies.1)
                        .cookie(cookies.2)
                        .insert_header((LOCATION, info.path().to_owned()))
                        .finish();
                }
            };

            let Ok(cookies) = cookies::from_token_response(tokens) else {
                tracing::error!("Error creating cookies");
                return HttpResponse::InternalServerError().finish();
            };

            return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                .cookie(cookies.0)
                .cookie(cookies.1)
                .cookie(cookies.2)
                .insert_header((LOCATION, info.path().to_owned()))
                .finish();
        }

        tracing::debug!("No tokens found, redirecting to Keycloak");

        let dest = app_data.keycloak.login_url(&info.to_url());
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((LOCATION, dest))
            .finish();
    };

    // Add the current path to the context
    let mut context = Context::new();
    context.insert("path", info.path());

    // Add the context to the request extensions
    if req.method() == "GET" {
        req.extensions_mut().insert(context);
    }

    // Validate the access token and execute the service call
    let access_token = access_token.clone();
    let tokens = JwtTokens::new(id_token.value().to_owned(), access_token.value().to_owned());

    match app_data.keycloak.validate_token(access_token.value()).await {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            req.extensions_mut().insert(tokens);
            next.run(req).await
        }
        Err(_) => {
            let dest = app_data.keycloak.login_url(&info.to_url());
            HttpResponse::build(StatusCode::FOUND)
                .insert_header((LOCATION, dest))
                .finish()
        }
    }
}
