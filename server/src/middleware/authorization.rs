use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{responses::HttpResponse, utils::extractor::Extractor};

pub async fn authorization(State(admin): State<bool>, req: Request, next: Next) -> Response {
    let user = match Extractor::claims(&req) {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let is_authorized = user.is_super_admin() || admin && user.is_admin(); // TODO: database check

    if !is_authorized {
        tracing::warn!("Unauthorized access attempt by user: {:?}", user.email());
        return HttpResponse::Forbidden().finish();
    }

    next.run(req).await
}
