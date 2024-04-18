pub struct Extractor;

impl Extractor {
    pub fn query((query_string, query_param): (&str, &str)) -> Option<String> {
        match query_string.split('&').find(|q| q.contains(query_param)) {
            Some(code) => {
                let code = code.split('=').collect::<Vec<&str>>()[1];
                Some(code.to_string())
            }
            None => None,
        }
    }

    pub fn uri(scheme: &str, host: &str, uri: &str) -> String {
        format!("{}://{}{}", scheme, host, uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::Uri;
    use rstest::*;

    #[rstest]
    #[case("param1=value1&code=abc123&param2=value2", "code", Some("abc123".to_owned()))]
    #[case("code=abc123", "code", Some("abc123".to_owned()))]
    #[case("param1=value1&param2=value2", "code", None)]
    #[case("", "value", None)]
    fn test_code_extraction_with_code(
        #[case] query: &str,
        #[case] param: &str,
        #[case] expected_code: Option<String>,
    ) {
        let extracted_code = Extractor::query((query, param));
        assert_eq!(extracted_code, expected_code);
    }

    #[rstest]
    #[case(
        "http",
        "localhost:8080",
        "/path/to/resource",
        "http://localhost:8080/path/to/resource"
    )]
    #[case(
        "https",
        "example.com",
        "/path/to/resource",
        "https://example.com/path/to/resource"
    )]
    #[case("http", "localhost:8080", "/", "http://localhost:8080/")]
    #[case("https", "example.com", "/", "https://example.com/")]
    #[case("http", "localhost:8080", "", "http://localhost:8080")]
    #[case("https", "example.com", "?param=value", "https://example.com/")]
    #[case("http", "localhost:8080", "?param=value", "http://localhost:8080/")]
    fn test_uri_extraction(
        #[case] scheme: &str,
        #[case] host: &str,
        #[case] path: &str,
        #[case] expected_uri: &str,
    ) {
        let uri = Uri::builder().path_and_query(path).build().unwrap();
        let extracted_uri = Extractor::uri(scheme, host, uri.path());
        assert_eq!(extracted_uri, expected_uri);
    }
}
