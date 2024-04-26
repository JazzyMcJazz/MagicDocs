use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;

use crate::utils::{extractor::Extractor, traits::Htmx};

pub struct Authorization {
    pub admin: bool,
}

impl<S, B> Transform<S, ServiceRequest> for Authorization
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthorizationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizationMiddleware {
            service: Rc::new(service),
            admin: self.admin,
        }))
    }
}

pub struct AuthorizationMiddleware<S> {
    service: Rc<S>,
    admin: bool,
}

impl<S, B> Service<ServiceRequest> for AuthorizationMiddleware<S>
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

        let user = claims.clone();
        let admin = self.admin;
        let srv = self.service.clone();

        Box::pin(async move {
            let is_authorized = user.is_super_admin() || admin && user.is_admin(); // TODO: database check

            if is_authorized {
                let fut = srv.call(req);
                return fut.await.map(ServiceResponse::map_into_left_body);
            }

            let (request, _) = req.into_parts();
            let (status, header) = request.redirect_status_and_header();

            let response = HttpResponse::build(status)
                .insert_header((header, "/"))
                .finish()
                .map_into_right_body();

            Ok(ServiceResponse::new(request, response))
        })
    }
}
