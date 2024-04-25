use std::{env, time::Duration};

use actix_web::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        Error,
    },
    rt::time::sleep,
    web::Bytes,
    HttpResponse,
};

// GET /
pub async fn sse() -> Result<HttpResponse, Error> {
    if env::var("RUST_ENV").unwrap_or_else(|_| "prod".to_string()) != "dev" {
        return Ok(HttpResponse::NotFound().finish());
    }

    let duration = Duration::from_secs(u64::MAX);
    let response_body = async_stream::stream! {
        loop {
            sleep(duration).await;
            yield Ok::<Bytes, Error>(Bytes::from(String::new()));
        }
    };

    let response = HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(response_body);

    Ok(response)
}
