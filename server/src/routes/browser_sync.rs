use std::{convert::Infallible, time::Duration};

use crate::{responses::HttpResponse, utils::config::Config};
use axum::response::{sse::Event, Response, Sse};
use futures_util::Stream;
use tokio::time::interval;

static CONNECTIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        CONNECTIONS.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        tracing::debug!(
            "Browser sync Connected: {}",
            CONNECTIONS.load(std::sync::atomic::Ordering::Relaxed)
        );
    }
}

// GET /
pub async fn sse() -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    let config = Config::default();
    if config.rust_env() != "dev" {
        return Err(HttpResponse::NotFound().finish());
    }

    CONNECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut interval = interval(Duration::from_secs(1));
    let stream = async_stream::stream! {
        tracing::debug!("Browser sync Connected: {}", CONNECTIONS.load(std::sync::atomic::Ordering::Relaxed));
        let _guard = Guard;
        loop {
            interval.tick().await;
            yield Ok(Event::default().data(""));
        }
    };

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    );

    Ok(sse)
}
