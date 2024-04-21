use std::{
    fmt,
    future::{ready, Ready},
};

use actix_web::{
    cookie::{self, Cookie},
    dev::{forward_ready, ConnectionInfo, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, StatusCode, Uri},
    web, Error, HttpMessage, HttpResponse, ResponseError,
};
use futures_util::future::LocalBoxFuture;
use tera::Context;

use crate::{
    keycloak::{GrantType, TokenResponse},
    server::AppState,
    utils::extractor::Extractor,
};

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let app_data = req.app_data::<web::Data<AppState>>().cloned();
        let conn_info = req.connection_info().clone();
        let uri = req.uri().clone();

        let query_string = req.query_string().to_owned();
        let kc_code = Extractor::query((&query_string, "code"));
        let redirect_uri = Extractor::uri(conn_info.scheme(), conn_info.host(), uri.path());

        let Some(app_data) = app_data else {
            return Box::pin(async move {
                Err(AuthenticationError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    token_response: None,
                    conn_info,
                    app_data,
                    uri,
                }
                .into())
            });
        };

        // Check if the user is redirected from Keycloak
        if let Some(code) = kc_code {
            let fut = self.service.call(req);

            return Box::pin(async move {
                let Ok(token) = app_data
                    .keycloak
                    .exchange_token(GrantType::AuthorizationCode(code.clone()), &redirect_uri)
                    .await
                else {
                    return Err(AuthenticationError {
                        status: StatusCode::UNAUTHORIZED,
                        app_data: Some(app_data),
                        token_response: None,
                        conn_info,
                        uri,
                    }
                    .into());
                };

                match Some(()) {
                    // This is a hack to make the code compile
                    Some(_) => Err(AuthenticationError {
                        status: StatusCode::FOUND,
                        token_response: Some(token),
                        app_data: Some(app_data),
                        conn_info,
                        uri,
                    }
                    .into()),
                    None => fut.await,
                }
            });
        }

        // Check if the user has a valid tokens or try to refresh the tokens
        let (Some(_), Some(access_token)) = (req.cookie("id"), req.cookie("ac")) else {
            if let Some(refresh_token) = req.cookie("rf") {
                let fut = self.service.call(req);

                return Box::pin(async move {
                    let Ok(token) = app_data
                        .keycloak
                        .exchange_token(
                            GrantType::RefreshToken(refresh_token.value().to_owned()),
                            &redirect_uri,
                        )
                        .await
                    else {
                        return Err(AuthenticationError {
                            status: StatusCode::UNAUTHORIZED,
                            app_data: Some(app_data),
                            token_response: None,
                            conn_info,
                            uri,
                        }
                        .into());
                    };

                    match Some(()) {
                        // This is a hack to make the code compile
                        Some(_) => Err(AuthenticationError {
                            status: StatusCode::FOUND,
                            token_response: Some(token),
                            app_data: Some(app_data),
                            conn_info,
                            uri,
                        }
                        .into()),
                        None => fut.await,
                    }
                });
            }

            return Box::pin(async move {
                Err(AuthenticationError {
                    status: StatusCode::UNAUTHORIZED,
                    app_data: Some(app_data),
                    token_response: None,
                    conn_info,
                    uri,
                }
                .into())
            });
        };

        // Add the current path to the context
        let mut context = Context::new();
        context.insert("path", req.path());

        // Add the context to the request extensions
        if req.method() == "GET" {
            req.extensions_mut().insert(context);
        }

        // The service call is not executed until the token is validated (at `fut.await`)
        let fut = self.service.call(req);

        // Validate the access token and execute the service call
        Box::pin(async move {
            match app_data.keycloak.validate_token(access_token.value()).await {
                Ok(_) => fut.await,
                Err(_) => Err(AuthenticationError {
                    status: StatusCode::UNAUTHORIZED,
                    app_data: Some(app_data),
                    token_response: None,
                    conn_info,
                    uri,
                }
                .into()),
            }
        })
    }
}

#[derive(Debug)]
pub struct AuthenticationError {
    app_data: Option<web::Data<AppState>>,
    conn_info: ConnectionInfo,
    status: StatusCode,
    uri: Uri,
    token_response: Option<TokenResponse>,
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unauthorized")
    }
}

impl ResponseError for AuthenticationError {
    fn error_response(&self) -> HttpResponse {
        match self.status {
            StatusCode::UNAUTHORIZED => self.unauthorized_response(),
            StatusCode::FOUND => self.found_response(),
            _ => self.internal_server_error_response(),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl AuthenticationError {
    fn found_response(&self) -> HttpResponse {
        let Some(tokens) = self.token_response.clone() else {
            return HttpResponse::InternalServerError().finish();
        };

        let Ok((id_cookie, access_cookie, refresh_cookie)) = CookieUtils::build_cookies(&tokens)
        else {
            return HttpResponse::InternalServerError().finish();
        };

        HttpResponse::Found()
            .cookie(id_cookie)
            .cookie(access_cookie)
            .cookie(refresh_cookie)
            .insert_header((header::LOCATION, self.uri.path()))
            .finish()
    }

    fn unauthorized_response(&self) -> HttpResponse {
        let Some(data) = &self.app_data else {
            return HttpResponse::InternalServerError().finish();
        };

        let scheme = self.conn_info.scheme();
        let host = self.conn_info.host();
        let uri = Extractor::uri(scheme, host, self.uri.path());
        let dest = data.keycloak.login_url(&uri);

        HttpResponse::Found()
            .append_header(("Location", dest))
            .finish()
    }

    fn internal_server_error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}

struct CookieUtils;

impl CookieUtils {
    // Helper function
    fn build_cookies(
        token_response: &'_ TokenResponse,
    ) -> Result<(Cookie<'_>, Cookie<'_>, Cookie<'_>), Box<dyn std::error::Error>> {
        let now = cookie::time::OffsetDateTime::now_utc().unix_timestamp();
        let expires = cookie::time::OffsetDateTime::from_unix_timestamp(
            now + token_response.expires_in() - 10,
        )?; // 10 seconds before the token expires
        let rf_expires =
            cookie::time::OffsetDateTime::from_unix_timestamp(now + 60 * 60 * 24 * 180)?; // 180 days

        let id_cookie = Cookie::build("id", token_response.id_token())
            .path("/")
            .secure(true)
            .http_only(true)
            .expires(expires)
            .finish();

        let access_cookie = Cookie::build("ac", token_response.access_token())
            .path("/")
            .secure(true)
            .http_only(true)
            .expires(expires)
            .finish();

        let refresh_cookie = Cookie::build("rf", token_response.refresh_token())
            .path("/")
            .secure(true)
            .http_only(true)
            .expires(rf_expires)
            .finish();

        Ok((id_cookie, access_cookie, refresh_cookie))
    }
}
