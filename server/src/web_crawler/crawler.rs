use reqwest::Url;
use anyhow::Result;
use tokio::time::sleep;

use super::spider::Spider;


pub struct Crawler {
    depth: i32,
    url: Url,
    queue: Vec<Url>,
    visited: Vec<Url>,
}

impl Crawler {
    pub fn new(url: String, depth: i32) -> Result<Self> {
        let url = Url::parse(&url)?;

        Ok(Self {
            depth,
            url: url.clone(),
            queue: vec![url],
            visited: vec![],
        })
    }

    pub async fn start(&mut self) {
        todo!(); // TODO: Check robots.txt

        while let Some(url) = self.queue.pop() {
            let spider = Spider::new(url.clone());
            let result = match spider.start().await {
                Ok(result) => result,
                Err(_) => continue,
            };

            for link in result.found_urls() {
                todo!(); // TODO: Account for depth
                if !self.visited.contains(&link) {
                    self.queue.push(link);
                }
            }

            if self.queue.is_empty() {
                break;
            }
            sleep(std::time::Duration::from_millis(500)).await;
        }
    }
}

pub struct CrawlerResult {

}