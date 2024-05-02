use std::collections::HashMap;

use regex::{self, Regex};
use reqwest::{header::USER_AGENT, Url};

use super::USER_AGENT_NAME;

pub struct RobotsTxt {
    rules: Vec<Rule>,
    delay: Option<u64>,
}

#[derive(Debug)]
enum Rule {
    Allow(String),
    Disallow(String),
    CrawlDelay(u64),
}

impl RobotsTxt {
    pub async fn from_url(url: &Url) -> Self {
        let body = Self::fetch(url).await;
        let rules = Self::parse(&body);
        let delay = rules.iter().find_map(|rule| match rule {
            Rule::CrawlDelay(delay) => Some(*delay),
            _ => None,
        });

        Self { rules, delay }
    }

    pub fn delay(&self) -> Option<u64> {
        self.delay
    }

    pub fn is_allowed<'a>(&'a self, url: &'a Url) -> bool {
        let path = url.path();
        let mut is_allowed = true;

        for rule in self.rules.iter() {
            match rule {
                Rule::Allow(rule) => {
                    let pattern = self.convert_pattern(rule);
                    match Regex::new(&pattern) {
                        Ok(regex) => {
                            if regex.is_match(path) {
                                is_allowed = true;
                            }
                        }
                        Err(_) => {
                            dbg!(rule);
                            continue;
                        }
                    }
                }
                Rule::Disallow(rule) => {
                    let pattern = self.convert_pattern(rule);
                    match Regex::new(&pattern) {
                        Ok(regex) => {
                            if regex.is_match(path) {
                                is_allowed = false;
                            }
                        }
                        Err(_) => {
                            dbg!(rule);
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        is_allowed
    }

    fn convert_pattern(&self, rule: &str) -> String {
        let mut pattern = regex::escape(rule);
        pattern = pattern.replace(r"\*", ".*");

        if pattern.ends_with(r"\$") {
            pattern.pop(); // Remove trailing $
            pattern.pop(); // Remove trailing \
            pattern.push('$'); // End-of-line anchor in regex
        }

        format!("^{pattern}")
    }

    async fn fetch(url: &Url) -> String {
        let Ok(url) = url.join("/robots.txt") else {
            return String::new();
        };

        let client = reqwest::Client::new();
        let request = client.get(url).header(USER_AGENT, USER_AGENT_NAME);

        let Ok(response) = request.send().await else {
            return String::new();
        };

        match response.status().is_success() {
            true => match response.text().await {
                Ok(body) => body,
                Err(_) => String::new(),
            },
            false => String::new(),
        }
    }

    fn parse(body: &str) -> Vec<Rule> {
        if body.is_empty() {
            return Vec::new();
        }

        let mut rules = HashMap::new();
        let mut current_user_agents = Vec::new();
        let mut last_was_user_agent = false;

        for line in body.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if line.starts_with("User-agent:") {
                if !last_was_user_agent {
                    current_user_agents.clear();
                }

                if let Some(user_agent) = line.split_whitespace().nth(1) {
                    current_user_agents.push(user_agent);
                    last_was_user_agent = true;
                };
            } else if line.starts_with("Disallow:") || line.starts_with("Allow:") {
                let allow = line.starts_with("Allow:");

                if let Some(rule) = line.split_whitespace().nth(1) {
                    for user_agent in current_user_agents.iter() {
                        rules
                            .entry(user_agent.to_string())
                            .or_insert(Vec::new())
                            .push(if allow {
                                Rule::Allow(rule.to_string())
                            } else {
                                Rule::Disallow(rule.to_string())
                            });
                    }
                }

                last_was_user_agent = false;
            } else if line.starts_with("Crawl-delay:") {
                if let Some(delay) = line.split_whitespace().nth(1) {
                    let Ok(delay) = delay.parse() else {
                        continue;
                    };

                    for user_agent in current_user_agents.iter() {
                        rules
                            .entry(user_agent.to_string())
                            .or_insert(Vec::new())
                            .push(Rule::CrawlDelay(delay));
                    }
                }

                last_was_user_agent = false;
            } else {
                last_was_user_agent = false;
            }
        }

        // Save * and USER_AGENT into a single vector
        let mut relevant_rules = Vec::new();
        relevant_rules.extend(rules.remove("*").unwrap_or_default());
        relevant_rules.extend(rules.remove(USER_AGENT_NAME).unwrap_or_default());

        relevant_rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::fs;

    #[rstest]
    #[case("bbc.com", "/ugc", false)]
    #[case("bbc.com", "/ugcisfine", true)]
    #[case("bbc.com", "/ugc/isnotfine", false)]
    #[case(
        "bbc.com",
        "/news/live/world-us-canada-68941518?src_origin=BBCS_BBC",
        true
    )]
    #[case("djangoproject.com", "/admin", false)]
    #[case("djangoproject.com", "/adminisnotfine", false)]
    #[case("djangoproject.com", "/admin/isnotfine", false)]
    #[case("djangoproject.com", "/anypath", true)]
    #[case("edition.cnn.com", "/somefile.jsx", false)]
    #[case("edition.cnn.com", "/somefile.jsx?some=param", false)]
    #[case("edition.cnn.com", "/NOKIA", false)]
    #[case("edition.cnn.com", "/NOKIAMORE", false)]
    #[case("edition.cnn.com", "/NOKIA/MORE", false)]
    #[case(
        "edition.cnn.com",
        "/business/live-news/university-protests-pro-palestinian-israel-05-02-24/index.html",
        true
    )]
    #[case("surrealdb.com", "/docs", true)]
    #[case("surrealdb.com", "/docs/surrealdb", true)]
    #[case("surrealdb.com", "/docs/surrealdb/surrealql/statements/break", true)]
    #[case("example.com", "/", false)]
    #[case("example.com", "/anypath", false)]
    #[case("example.com", "/anyfile.html", false)]
    #[case("example.com", "/anyfile.jsx", false)]
    fn test_process_url(#[case] domain: &str, #[case] path: &str, #[case] expected: bool) {
        let body =
            fs::read_to_string(format!("src/web_crawler/test_files/{domain}.robots.txt")).unwrap();
        let rules = RobotsTxt::parse(&body);
        let robots = RobotsTxt { rules, delay: None };
        let url = Url::parse(format!("https://{domain}{path}").as_str()).unwrap();

        let is_allowed = robots.is_allowed(&url);

        assert_eq!(is_allowed, expected);
    }
}
