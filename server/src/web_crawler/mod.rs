pub mod crawler;
mod robots_txt;
mod spider;

static USER_AGENT_NAME: &str = "MagicDocsBot";

#[cfg(test)]
mod tests {
    use crate::web_crawler::crawler::StreamOutput;

    use super::crawler::Crawler;
    use futures_util::{pin_mut, StreamExt};
    use mockito::Server;
    use tokio::test;

    #[test]
    async fn test_crawler_with_no_max_depth() {
        let mut server = Server::new_async().await;
        let url = server.url();

        let robots = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/")
            .with_body("<html><body><a href='/one'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/two'>Two</a></body></html>")
            .create_async()
            .await;

        let mut crawler = Crawler::new(url, None).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();

        assert_eq!(results.len(), 2);
    }

    #[test]
    async fn test_crawler_with_max_depth_zero() {
        let mut server = Server::new_async().await;
        let url = server.url();

        let robots = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/")
            .with_body("<html><body><a href='/one'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/two'>Two</a></body></html>")
            .create_async()
            .await
            .expect(0);

        let mut crawler = Crawler::new(url, Some(0)).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();

        assert_eq!(results.len(), 1);
    }

    #[test]
    async fn test_crawler_with_max_depth_one() {
        let mut server = Server::new_async().await;
        let url = server.url();

        let robots = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/")
            .with_body("<html><body><a href='/one'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/one/two'>Two</a></body></html>")
            .create_async()
            .await;

        let m3 = server
            .mock("GET", "/one/two")
            .with_body("<html><body><a href='/three'>Three</a></body></html>")
            .create_async()
            .await
            .expect(0);

        let mut crawler = Crawler::new(url, Some(1)).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();
        m3.assert();

        assert_eq!(results.len(), 2);
    }

    #[test]
    async fn test_crawler_with_max_depth_two() {
        let mut server = Server::new_async().await;
        let url = server.url();

        let robots = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/")
            .with_body("<html><body><a href='/one'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/one/two'>Two</a></body></html>")
            .create_async()
            .await;

        let m3 = server
            .mock("GET", "/one/two")
            .with_body("<html><body><a href='/one/two/three'>Three</a></body></html>")
            .create_async()
            .await;

        let m4 = server
            .mock("GET", "/one/two/three")
            .with_body("<html><body><a href='/one/two/three/four'>Four</a></body></html>")
            .create_async()
            .await
            .expect(0);

        let mut crawler = Crawler::new(url, Some(2)).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();
        m3.assert();
        m4.assert();

        assert_eq!(results.len(), 3);
    }

    #[test]
    async fn test_crawler_does_not_visit_neighbors() {
        let mut server = Server::new_async().await;
        let url = format!("{}/one", server.url());

        let robots = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/two'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/two")
            .with_body("<html><body><a href='/two'>Two</a></body></html>")
            .create_async()
            .await
            .expect(0);

        let mut crawler = Crawler::new(url, None).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();

        assert_eq!(results.len(), 1);
    }

    #[test]
    async fn test_crawler_respects_robots_txt() {
        let mut server = Server::new_async().await;
        let url = server.url();

        let robots = server
            .mock("GET", "/robots.txt")
            .with_body("User-agent: *\nDisallow: /one")
            .create_async()
            .await;

        let m1 = server
            .mock("GET", "/")
            .with_body("<html><body><a href='/one'>One</a></body></html>")
            .create_async()
            .await;

        let m2 = server
            .mock("GET", "/one")
            .with_body("<html><body><a href='/two'>Two</a></body></html>")
            .create_async()
            .await
            .expect(0);

        let mut crawler = Crawler::new(url, None).unwrap();
        let stream = crawler.start().await;
        pin_mut!(stream);
        let mut results = vec![];
        while let Some(output) = stream.next().await {
            match output {
                StreamOutput::Result(result) => results.push(result),
                _ => {}
            }
        }

        robots.assert();
        m1.assert();
        m2.assert();

        assert_eq!(results.len(), 1);
    }
}
