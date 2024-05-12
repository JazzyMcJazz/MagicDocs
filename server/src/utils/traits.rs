use axum::{
    http::{HeaderName, StatusCode},
    response::Response,
};
use http::{header::LOCATION, HeaderMap};

use crate::responses::HttpResponse;

pub trait Htmx {
    fn is_htmx(&self) -> bool;
    fn redirect_status_and_header(&self) -> (StatusCode, HeaderName) {
        if self.is_htmx() {
            (StatusCode::OK, HeaderName::from_static("hx-redirect"))
        } else {
            (StatusCode::FOUND, LOCATION)
        }
    }
}

impl Htmx for HeaderMap {
    fn is_htmx(&self) -> bool {
        match self.get("HX-Request") {
            Some(header) => header == "true",
            None => false,
        }
    }
}

pub trait TryRender {
    fn try_render(&self, template: &str, context: &tera::Context) -> Response;
}

impl TryRender for tera::Tera {
    fn try_render(&self, template: &str, context: &tera::Context) -> Response {
        let html = match self.render(template, context) {
            Ok(html) => html,
            Err(e) => {
                tracing::error!("Failed to render template: {:?}", e);
                return HttpResponse::InternalServerError()
                    .body("Template error")
                    .finish();
            }
        };

        HttpResponse::Ok().body(html)
    }
}
