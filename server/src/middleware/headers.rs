use axum::{extract::Request, middleware::Next, response::Response};
use http::{
    header::{CACHE_CONTROL, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS},
    HeaderValue,
};

pub async fn default(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;

    res.headers_mut().extend([
        (X_FRAME_OPTIONS, HeaderValue::from_static("DENY")),
        (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
    ]);

    res
}

pub async fn cache_control(req: Request, next: Next) -> Response {
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "prod".to_string());
    let cache_control = if rust_env == "dev" {
        "no-store"
        // "max-age=600"
    } else {
        "max-age=600"
    };

    let mut res = next.run(req).await;
    res.headers_mut()
        .append(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    res
}
