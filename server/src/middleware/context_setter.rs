use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use tera::Context;

use crate::utils::{context_data::UserData, extractor::Extractor};

pub struct ContextSetter;

impl<S, B> Transform<S, ServiceRequest> for ContextSetter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ContextSetterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContextSetterMiddleware { service }))
    }
}

pub struct ContextSetterMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContextSetterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let claims = match Extractor::extract_claims(&req) {
            Ok(claims) => claims,
            Err(_) => {
                let (request, _) = req.into_parts();
                let response = HttpResponse::InternalServerError()
                    .finish()
                    .map_into_right_body();

                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
        };

        let user_data = UserData::from_claims(&claims);
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "prod".to_string());

        let mut context = Context::new();
        context.insert("path", req.path());
        context.insert("user", &user_data);
        context.insert("env", &env);

        // Add the context to the request extensions
        req.extensions_mut().insert(context);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) })
    }
}
