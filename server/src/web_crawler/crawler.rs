use std::collections::HashSet;

use anyhow::Result;
use reqwest::Url;
use tokio::time::sleep;

use super::{
    robots_txt::RobotsTxt,
    spider::{Spider, SpiderResult},
};

pub struct Crawler {
    max_depth: Option<usize>,
    url: Url,
}

impl Crawler {
    pub fn new(url: String, max_depth: Option<usize>) -> Result<Self> {
        let url = Url::parse(&url)?;

        Ok(Self {
            max_depth,
            url: url.clone(),
        })
    }

    pub async fn start(&mut self) -> Result<Vec<CrawlerResult>> {
        let robots = RobotsTxt::from_url(&self.url).await;
        let delay = robots.delay().unwrap_or(500);
        let mut results = vec![];

        let mut queue: Vec<Url> = vec![self.url.clone()];
        let mut visited: HashSet<String> = HashSet::new();

        while let Some(url) = queue.pop() {
            if !robots.is_allowed(&url) {
                continue;
            }

            let path = url.path().to_owned();

            if visited.contains(&path) {
                continue;
            } else {
                visited.insert(path.to_owned());
            }

            println!(
                "Crawling: {}://{}{}",
                url.scheme(),
                url.host_str().unwrap(),
                url.path()
            );
            let spider = Spider::new(url.clone());
            let result = match spider.start().await {
                Ok(result) => result,
                Err(_) => continue,
            };

            for link in result.found_urls() {
                let path = link.path().to_owned();
                let path = path.trim_end_matches('/');

                if visited.contains(path) {
                    continue;
                }

                let relative_depth = self.find_relative_depth(&link);
                let max_depth = self.max_depth.unwrap_or(usize::MAX);

                if 0 < relative_depth && relative_depth <= max_depth {
                    queue.push(link);
                } else {
                    visited.insert(path.to_owned());
                }
            }

            results.push(result);

            if queue.is_empty() {
                break;
            }
            sleep(std::time::Duration::from_millis(delay)).await;
        }

        Ok(CrawlerResult::from_spider_results(results))
    }

    fn find_relative_depth(&self, url: &Url) -> usize {
        let normalised_base_path = self.url.path().trim_end_matches('/');
        let normalised_path = url.path().trim_end_matches('/');

        if normalised_base_path == normalised_path {
            return 0;
        }

        if normalised_path.starts_with(normalised_base_path) {
            return normalised_path.matches('/').count()
                - normalised_base_path.matches('/').count();
        }

        0
    }
}

pub struct CrawlerResult {
    path: String,
    title: String,
    html: String,
}

impl CrawlerResult {
    fn from_spider_results(spider_results: Vec<SpiderResult>) -> Vec<Self> {
        let mut results = vec![];

        for spider_result in spider_results {
            results.push(Self {
                path: spider_result.url().path().to_owned(),
                title: spider_result.page_title(),
                html: spider_result.html(),
            });
        }
        results
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn html(&self) -> String {
        self.html.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("https://example.com", "/", 0)]
    #[case("https://example.com/one/two", "/one/two", 0)]
    #[case("https://example.com", "/one", 1)]
    #[case("https://example.com/one/two", "/one/two/three", 1)]
    #[case("https://example.com/one/two", "/one/two/three/four", 2)]
    #[case("https://example.com/one/two", "/one/two/three/five", 2)]
    #[case("https://example.com/one/two", "/one/two/three/five/six", 3)]
    #[case("https://example.com/one", "/", 0)]
    #[case("https://example.com/one", "/two", 0)]
    #[case("https://example.com/two/", "/one/two/three", 0)]
    #[case("https://example.com/two/three", "/one/two/three", 0)]
    #[case("https://example.com/two/three", "/one/two/three/four", 0)]
    #[case("https://example.com/two/three", "/one/two/three/five", 0)]
    fn test_relative_depth(#[case] url: &str, #[case] other_path: &str, #[case] expected: usize) {
        let crawler = Crawler::new(url.to_string(), None).unwrap();

        let scheme = crawler.url.scheme();
        let host = crawler.url.host_str().unwrap();
        let url_string = format!("{scheme}://{host}{other_path}");

        let other_url = Url::parse(url_string.as_str()).unwrap();
        let relative_depth = crawler.find_relative_depth(&other_url);

        assert_eq!(relative_depth, expected);
    }
}
