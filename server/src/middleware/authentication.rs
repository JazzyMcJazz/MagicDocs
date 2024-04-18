use std::{
    fmt,
    future::{ready, Ready},
};

use actix_web::{
    dev::{forward_ready, ConnectionInfo, Service, ServiceRequest, ServiceResponse, Transform},
    http::{StatusCode, Uri},
    web, Error, HttpMessage, HttpResponse, ResponseError,
};
use futures_util::future::LocalBoxFuture;
use tera::Context;

use crate::{server::AppState, utils::extractor::Extractor};

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
                match app_data
                    .keycloak
                    .exchange_code(&code.clone(), &redirect_uri)
                    .await
                {
                    Ok(token) => {
                        dbg!(token);
                        fut.await
                    }
                    Err(e) => {
                        dbg!(e);
                        Err(AuthenticationError {
                            status: StatusCode::UNAUTHORIZED,
                            app_data: Some(app_data),
                            conn_info,
                            uri,
                        }
                        .into())
                    }
                }
            });
        } else {
            let Some(_) = req.cookie("id") else {
                let b = Box::pin(async move {
                    Err(AuthenticationError {
                        status: StatusCode::UNAUTHORIZED,
                        app_data: Some(app_data),
                        conn_info,
                        uri,
                    }
                    .into())
                });
                return b;
            };
        }

        // Add the next path to the context (if it exists)
        // req.query_string().split('&').for_each(|q| {
        //     if q.contains("next=") {
        //         context.insert("next", q.split('=').last().unwrap_or("/"));
        //     }
        // });

        // Add the current path to the context
        let mut context = Context::new();
        context.insert("path", req.path());
        context.insert("next", "/");

        // Add the context to the request extensions
        if req.method() == "GET" {
            req.extensions_mut().insert(context);
        }

        let fut = self.service.call(req);
        Box::pin(fut)
    }
}

#[derive(Debug)]
pub struct AuthenticationError {
    app_data: Option<web::Data<AppState>>,
    conn_info: ConnectionInfo,
    status: StatusCode,
    uri: Uri,
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
            _ => self.internal_server_error_response(),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

impl AuthenticationError {
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
