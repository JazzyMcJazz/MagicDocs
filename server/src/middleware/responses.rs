use actix_web::{
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    HttpResponse,
};

pub enum ErrorResponse {
    _BadRequest,
    _NotFound,
    InternalServerError,
}

impl ErrorResponse {
    pub fn build<B>(&self, req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        match self {
            ErrorResponse::_BadRequest => self.builder(req, StatusCode::BAD_REQUEST),
            ErrorResponse::_NotFound => self.builder(req, StatusCode::NOT_FOUND),
            ErrorResponse::InternalServerError => {
                self.builder(req, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    fn builder<B>(&self, req: ServiceRequest, code: StatusCode) -> ServiceResponse<EitherBody<B>> {
        let (request, _) = req.into_parts();
        let response = HttpResponse::build(code).finish().map_into_right_body();

        ServiceResponse::new(request, response)
    }
}
