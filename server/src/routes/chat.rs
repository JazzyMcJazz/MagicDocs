use std::{convert::Infallible, time::Duration};

use crate::{
    langchain::{LLMOutput, LLMProvider, Langchain},
    models::{ChatForm, Slugs},
    responses::HttpResponse,
    server::AppState,
};
use axum::{
    extract::{Path, State},
    response::{sse::Event, IntoResponse, Response, Sse},
    Form,
};
use futures_util::StreamExt;
use http::HeaderValue;
use tokio::pin;

// GET /
pub async fn chat(
    State(data): State<AppState>,
    Path(path): Path<Slugs>,
    Form(form): Form<ChatForm>,
) -> Response {
    let Some(project_id) = path.project_id() else {
        return HttpResponse::BadRequest().finish();
    };
    let Some(version) = path.version() else {
        return HttpResponse::BadRequest().finish();
    };

    let stream = async_stream::stream! {
        let db = &data.conn;
        let lc = Langchain::new(LLMProvider::OpenAI);
        let prompt = form.message;

        let stream = match lc.chat_completion(
            db,
            project_id,
            version,
            &prompt) {
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
                    match output {
                        LLMOutput::Content(content) => {
                            yield Ok::<_, Infallible>(Event::default().data(content));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                }
            }
        }

    };

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    );

    let mut res = sse.into_response();
    res.headers_mut()
        .insert("X-Accel-Buffering", HeaderValue::from_static("no"));
    res
}
