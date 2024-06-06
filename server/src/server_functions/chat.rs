use leptos::{
    server,
    server_fn::codec::{StreamingText, TextStream},
    ServerFnError,
};

#[server(output = StreamingText)]
pub async fn chat(
    project_id: i32,
    version: i32,
    prompt: String,
) -> Result<TextStream, ServerFnError> {
    use crate::{
        langchain::{LLMOutput, LLMProvider, Langchain},
        server::AppState,
    };
    use futures_util::StreamExt;
    use leptos::use_context;
    use tokio::pin;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let stream = async_stream::stream! {
        let db = &state.conn;
        let lc = Langchain::new(LLMProvider::OpenAI);

        let stream = match lc.chat_completion(
            db,
            project_id,
            version,
            &prompt
        ).await {
            Ok(stream) => stream,
            Err(e) => {
                let err = format!("Error: {:?}", e);
                tracing::error!("{}", &err);
                yield Err::<_, ServerFnError>(ServerFnError::ServerError(err));
                return;
            }
        };

        pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                Ok(output) => {
                    match output {
                        LLMOutput::Content(content) => {
                            yield Ok::<_, ServerFnError>(format!("data: {content}¤¤"));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                }
            }
        }
    };

    Ok(TextStream::new(stream))
}
