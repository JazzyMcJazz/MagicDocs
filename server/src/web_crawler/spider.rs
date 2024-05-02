use anyhow::{bail, Result};
use reqwest::{
    header::{HeaderValue, CONTENT_TYPE, USER_AGENT},
    Url,
};
use scraper::{Html, Selector};

use super::USER_AGENT_NAME;

static EXPECTED_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("text/html");

pub struct Spider {
    url: Url,
    client: reqwest::Client,
}

impl Spider {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn start(&self) -> Result<SpiderResult> {
        let html = self.fetch_html(&self.url).await?;
        let links = self.extract_urls(&html)?;
        let title = self.extract_title(&html)?;
        let result = SpiderResult::new(self.url.clone(), links, title, html);
        Ok(result)
    }

    async fn fetch_html(&self, url: &Url) -> Result<String> {
        let req = self
            .client
            .get(url.clone())
            .header(USER_AGENT, USER_AGENT_NAME);
        let res = req.send().await?;

        if !res.status().is_success() {
            bail!("Failed to fetch page");
        }

        let content_type_html = match res.headers().get(CONTENT_TYPE) {
            Some(content_type) => content_type.eq(&EXPECTED_CONTENT_TYPE),
            None => false,
        };

        let html = res.text().await?;

        let html = match content_type_html {
            false => self.sniff_html_content(html)?,
            true => html,
        };

        Ok(html)
    }

    fn sniff_html_content(&self, html: String) -> Result<String> {
        let lower_content = html.to_lowercase();
        let is_html =
            lower_content.starts_with("<!doctype html>") || lower_content.starts_with("<html");
        if !is_html {
            bail!("Content is not HTML");
        }
        Ok(html)
    }

    fn extract_urls(&self, html: &str) -> Result<Vec<Url>> {
        let document = Html::parse_document(html);
        let Ok(selector) = Selector::parse("a[href]") else {
            bail!("Failed to parse selector");
        };

        let links: Vec<Url> = document
            .select(&selector)
            .filter_map(|e| e.value().attr("href"))
            .filter_map(|link| self.parse_link(link).ok())
            .collect();

        Ok(links)
    }

    fn extract_title(&self, html: &str) -> Result<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("h1").unwrap();

        let mut title = document
            .select(&selector)
            .next()
            .map(|e| e.text().collect::<String>());

        if title.is_none() {
            let selector = Selector::parse("title").unwrap();
            title = document
                .select(&selector)
                .next()
                .map(|e| e.text().collect::<String>());
        }

        Ok(title.unwrap_or("unnamed".to_string()))
    }

    fn parse_link(&self, link: &str) -> Result<Url> {
        match Url::parse(link) {
            Ok(parsed_url) => {
                if parsed_url.host_str() != self.url.host_str() {
                    bail!("Link is not from the same domain");
                }

                Ok(parsed_url)
            }
            Err(_) => {
                let base = self.url.clone();
                match base.join(link) {
                    Ok(parsed_url) => Ok(parsed_url),
                    Err(_) => bail!("Failed to parse link"),
                }
            }
        }
    }
}

pub struct SpiderResult {
    url: Url,
    found_urls: Vec<Url>,
    page_title: String,
    html: String,
}

impl SpiderResult {
    fn new(url: Url, links: Vec<Url>, page_title: String, html: String) -> Self {
        Self {
            url,
            found_urls: links,
            page_title,
            html,
        }
    }
    pub fn url(&self) -> Url {
        self.url.clone()
    }
    pub fn found_urls(&self) -> Vec<Url> {
        self.found_urls.clone()
    }
    pub fn page_title(&self) -> String {
        self.page_title.clone()
    }
    pub fn html(&self) -> String {
        self.html.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use mockito::*;
    use std::fs;

    #[test]
    async fn test_spider() {
        let mut server = Server::new_async().await;

        let host = server.url();
        let url = Url::parse(host.as_str()).unwrap();

        let body = fs::read_to_string("src/web_crawler/test_files/rust-lang.html").unwrap();
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let spider = Spider::new(url.clone());
        let result = spider.start().await.unwrap();

        mock.assert();

        assert_eq!(&result.url(), &url);
        assert_eq!(result.page_title(), "Rust");
        assert!(!result.found_urls().is_empty());
        assert!(!result.html().is_empty());

        for found_url in result.found_urls() {
            assert_eq!(found_url.host_str(), url.host_str());
        }
    }
}
