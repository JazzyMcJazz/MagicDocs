use std::{convert::Infallible, time::Duration};

use crate::{
    langchain::{LLMProvider, Langchain},
    models::ChatForm,
};
use axum::{
    response::{sse::Event, Response, Sse},
    Form,
};
use futures_util::{Stream, StreamExt};
use tokio::pin;

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        tracing::warn!("Chat cancelled. Cleanup not implemented",);
    }
}

// GET /
pub async fn chat(
    Form(form): Form<ChatForm>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    // TODO: Implement authorization
    // if not_authorized {
    //     return Err(HttpResponse::Forbidden().finish());
    // }

    let stream = async_stream::stream! {
        tracing::info!("Chat started");
        let _guard = Guard;
        let lc = Langchain::new(LLMProvider::OpenAI);
        let prompt = form.message;

        let stream = match lc.chat_completion(&prompt).await {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                return;
            }
        };

        pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                Ok(output) => {
                    let content = output.content().unwrap_or_default();
                    yield Ok(Event::default().data(content));
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                }
            }
        }

    };

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    );

    Ok(sse)
}
