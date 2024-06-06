use super::models::Document;
use leptos::{
    server,
    server_fn::codec::{StreamingText, TextStream},
    ServerFnError,
};

#[server]
pub async fn get_document(
    project_id: i32,
    version: i32,
    document_id: i32,
) -> Result<Document, ServerFnError> {
    use crate::{database::Repo, markdown::Markdown, server::AppState};
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        tracing::error!("Failed to get app state");
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let Ok(Some(document)) = db
        .documents()
        .find_by_id_and_version(project_id, version, document_id)
        .await
    else {
        tracing::error!("Failed to get document");
        return Err(ServerFnError::ServerError(
            "Failed to get document".to_string(),
        ));
    };

    let document = Document {
        id: document.id,
        name: document.name,
        content: Markdown::to_html(&document.content),
        source: document.source,
    };

    Ok(document)
}

#[server]
pub async fn create_document(
    project_id: i32,
    title: String,
    content: String,
) -> Result<i32, ServerFnError> {
    use crate::{database::Repo, server::AppState};
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    tracing::info!("Creating document for project {}", project_id);
    let db = &state.conn;

    let Ok((document_id, _)) = db.documents().create(project_id, &title, &content).await else {
        return Err(ServerFnError::ServerError(
            "Failed to create document".to_string(),
        ));
    };

    Ok(document_id)
}

#[server(output = StreamingText)]
pub async fn crawl_website(
    project_id: i32,
    url: String,
    max_depth: Option<usize>,
) -> Result<TextStream, ServerFnError> {
    use crate::{
        database::Repo,
        parsing::HtmlParser,
        server::AppState,
        web_crawler::crawler::{Crawler, StreamOutput},
    };
    use futures_util::{pin_mut, StreamExt};
    use http::header::{HeaderName, HeaderValue};
    use leptos::{expect_context, use_context};
    use leptos_axum::ResponseOptions;
    use std::str::FromStr;
    use tokio::time::sleep;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let response = expect_context::<ResponseOptions>();

    let stream = async_stream::stream! {
        let db = &state.conn;

        let mut results = vec![];
        let Ok(mut crawler) = Crawler::new(url, max_depth) else {
            yield Ok::<_, ServerFnError>("Error".to_owned());
            return;
        };

        let stream = crawler.start().await;
        pin_mut!(stream);

        while let Some(r) = stream.next().await {
            match r {
                StreamOutput::Message(msg) => {
                    yield Ok::<_, ServerFnError>(msg);
                }
                StreamOutput::Result(res) => {
                    results.push(res);
                }
            }
        };

        yield Ok("Processings results...".to_owned());

        let mut documents = vec![];
        for res in results {
            let message = format!("Processing {}", res.title());
            yield Ok(message);
            let parser = HtmlParser::new(&res.title(), &res.html(), res.url());
            match parser.parse() {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    let message = format!("Error: {}", e);
                    yield Ok(message);
                }
            }
            sleep(std::time::Duration::from_millis(100)).await;
        }

        yield Ok("Saving documents...".to_owned());

        let _ = db.documents().create_many_from_documents(project_id, documents).await;

        yield Ok("Done".to_owned());
        sleep(std::time::Duration::from_secs(1)).await;
    };

    if let Ok(key) = HeaderName::from_str("X-Accel-Buffering") {
        let value = HeaderValue::from_static("no");
        response.insert_header(key, value);
    }

    Ok(TextStream::new(stream))
}

#[server]
pub async fn delete_document(
    project_id: i32,
    project_version: i32,
    document_id: i32,
) -> Result<(), ServerFnError> {
    use crate::{database::Repo, server::AppState};
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = &state.conn;

    let _ = match db
        .documents()
        .delete(document_id, project_id, project_version)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error deleting document: {:?}", e);
            return Err(ServerFnError::ServerError(
                "Failed to delete document".to_string(),
            ));
        }
    };

    Ok(())
}
