use anyhow::{bail, Result};
use llm_chain::traits::Embeddings;
use llm_chain_openai::embeddings;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tiktoken_rs::cl100k_base;

use crate::models::Embedding;

pub async fn embed(content: &str) -> Result<Vec<Embedding>> {
    let embeddings = embeddings::Embeddings::default();

    let tokenizer = cl100k_base()?;
    let max_characters = 500..2000;
    let chunk_config = ChunkConfig::new(max_characters).with_sizer(tokenizer);
    let splitter = MarkdownSplitter::new(chunk_config);

    let chunks = splitter.chunks(content);
    let texts = chunks
        .into_iter()
        .map(|chunk| chunk.to_owned())
        .collect::<Vec<_>>();

    let embedded_vecs = match embeddings.embed_texts(texts.to_owned()).await {
        Ok(embedded_vecs) => embedded_vecs,
        Err(e) => {
            tracing::error!("Failed to embed texts: {:?}", e);
            bail!("Failed to embed texts: {:?}", e);
        }
    };

    let embeddings = texts
        .into_iter()
        .zip(embedded_vecs.into_iter())
        .map(|(text, vec)| Embedding::new(text, vec))
        .collect::<Vec<_>>();

    Ok(embeddings)
}
