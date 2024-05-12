use anyhow::Result;
use reqwest::Url;

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

        let result = html2md::parse_html(&content);

        Ok(HtmlParser {
            name: self.name,
            content: result,
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
