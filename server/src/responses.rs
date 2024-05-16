use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use http::header::{IntoHeaderName, CONTENT_TYPE};

pub struct HttpResponse;

#[derive(Default)]
pub struct HttpResponseBuilder {
    status: StatusCode,
    body: String,
    headers: HeaderMap,
    cookies: Vec<Cookie<'static>>,
}

impl HttpResponse {
    #[allow(non_snake_case)]
    pub fn Ok() -> Self {
        Self
    }
    #[allow(non_snake_case)]
    pub fn BadRequest() -> HttpResponseBuilder {
        Self::build(StatusCode::BAD_REQUEST)
    }
    #[allow(non_snake_case)]
    pub fn Forbidden() -> HttpResponseBuilder {
        Self::build(StatusCode::FORBIDDEN)
    }
    #[allow(non_snake_case)]
    pub fn NotFound() -> HttpResponseBuilder {
        Self::build(StatusCode::NOT_FOUND)
    }
    #[allow(non_snake_case)]
    pub fn MethodNotAllowed() -> HttpResponseBuilder {
        Self::build(StatusCode::METHOD_NOT_ALLOWED)
    }
    #[allow(non_snake_case)]
    pub fn InternalServerError() -> HttpResponseBuilder {
        Self::build(StatusCode::INTERNAL_SERVER_ERROR)
    }
    pub fn build(status: StatusCode) -> HttpResponseBuilder {
        HttpResponseBuilder {
            status,
            ..Default::default()
        }
    }
}

impl HttpResponse {
    pub fn body<S: AsRef<str>>(self, body: S) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));

        (StatusCode::OK, headers, body.as_ref().to_string()).into_response()
    }
}

impl HttpResponseBuilder {
    pub fn body<S: AsRef<str>>(mut self, body: S) -> Self {
        self.body = body.as_ref().to_string();
        self
    }

    pub fn insert_header<N>(mut self, header: (N, String)) -> Self
    where
        N: IntoHeaderName,
    {
        let value = match HeaderValue::from_str(&header.1) {
            Ok(value) => value,
            Err(_) => return self,
        };

        self.headers.insert(header.0, value);
        self
    }

    pub fn cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }

    pub fn finish(mut self) -> Response {
        let mut jar = CookieJar::new();
        for cookie in self.cookies {
            jar = jar.add(cookie);
        }

        if self.status.is_success() {
            self.headers
                .insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        }

        (self.status, jar, self.headers, self.body).into_response()
    }
}
