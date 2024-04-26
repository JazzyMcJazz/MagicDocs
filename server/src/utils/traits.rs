use actix_web::{http::StatusCode, HttpMessage};

pub trait Htmx {
    fn is_htmx(&self) -> bool;
    fn redirect_status_and_header(&self) -> (StatusCode, &str) {
        if self.is_htmx() {
            (StatusCode::OK, "HX-Redirect")
        } else {
            (StatusCode::FOUND, "LOCATION")
        }
    }
}

impl<T: HttpMessage> Htmx for T {
    fn is_htmx(&self) -> bool {
        match self.headers().get("HX-Request") {
            Some(header) => header == "true",
            None => false,
        }
    }
}
