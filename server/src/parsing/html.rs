use anyhow::{bail, Result};
use reqwest::Url;
use scraper::{Html, Selector};

pub struct Waiting;
pub struct Done;

pub struct HtmlParser<State = Waiting> {
    name: String,
    content: String,
    source: String,
    state: std::marker::PhantomData<State>,
}

impl HtmlParser {
    pub fn new(name: &str, html: &str, url: Url) -> Self {
        Self {
            name: name.to_string(),
            content: html.to_string(),
            source: format!("{}", url),
            state: Default::default(),
        }
    }
}

impl HtmlParser<Waiting> {
    pub fn parse(self) -> Result<HtmlParser<Done>> {
        let content = self.content;
        let source = self.source;

        let Ok(selector) = Selector::parse("a") else {
            bail!("Failed to parse {}", self.name)
        };
        let mut document = Html::parse_document(&content);
        let node_ids: Vec<_> = document.select(&selector).map(|node| node.id()).collect();
        for id in node_ids {
            match document.tree.get_mut(id) {
                Some(mut node) => {
                    node.detach();
                }
                None => {
                    bail!("Failed to parse {}", self.name);
                }
            }
        }

        let content = html2text::from_read(document.html().as_bytes(), 80);

        Ok(HtmlParser {
            name: self.name,
            content,
            source,
            state: Default::default(),
        })
    }
}

impl HtmlParser<Done> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}
