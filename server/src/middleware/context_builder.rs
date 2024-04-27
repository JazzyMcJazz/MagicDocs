use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use tera::Context;

use crate::{
    database::Repo,
    server::AppState,
    utils::{context_data::UserData, extractor::Extractor},
};

pub struct ContextBuilder;

impl<S, B> Transform<S, ServiceRequest> for ContextBuilder
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ContextBuilderMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContextBuilderMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ContextBuilderMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ContextBuilderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let Some(app_data) = req.app_data::<web::Data<AppState>>().cloned() else {
            let res = internal_server_error_response(req);
            return Box::pin(async { Ok(res) });
        };

        let Ok(claims) = Extractor::extract_claims(&req) else {
            let res = internal_server_error_response(req);
            return Box::pin(async { Ok(res) });
        };

        let user_data = UserData::from_claims(&claims);
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "prod".to_string());

        let srv = self.service.clone();
        Box::pin(async move {
            let db = app_data.conn.to_owned();
            let projects = match db.projects().all(&user_data).await {
                Ok(projects) => projects,
                Err(_) => Vec::new(),
            };

            let mut context = Context::new();
            context.insert("path", req.path());
            context.insert("user", &user_data);
            context.insert("env", &env);
            context.insert("projects", &projects);

            req.extensions_mut().insert(context);
            let fut = srv.call(req).await;
            fut.map(ServiceResponse::map_into_left_body)
        })
    }
}

fn internal_server_error_response<B>(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
    let (request, _) = req.into_parts();
    let response = HttpResponse::InternalServerError()
        .finish()
        .map_into_right_body();

    ServiceResponse::new(request, response)
}
