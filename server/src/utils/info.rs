use axum::extract::Request;
use http::{
    header::{FORWARDED, HOST},
    HeaderMap, HeaderName,
};

static X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");
static X_FORWARDED_HOST: HeaderName = HeaderName::from_static("x-forwarded-host");
static X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");

fn unquote(val: &str) -> &str {
    val.trim().trim_start_matches('"').trim_end_matches('"')
}

fn first_header_value<'a>(req: &'a HeaderMap, name: &'_ HeaderName) -> Option<&'a str> {
    let hdr = req.get(name)?.to_str().ok()?;
    let val = hdr.split(',').next()?.trim();
    Some(val)
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionInfo {
    host: String,
    scheme: String,
    path: String,
    query: Option<String>,
    _real_remote_addr: Option<String>,
}

impl ConnectionInfo {
    pub fn new(req: &Request) -> ConnectionInfo {
        let headers = req.headers();

        let mut host = None;
        let mut scheme = None;
        let mut realip_remote_addr = None;

        for (name, val) in headers
            .get_all(&FORWARDED)
            .iter()
            .filter_map(|hdr| hdr.to_str().ok())
            .flat_map(|val| val.split(';'))
            .flat_map(|vals| vals.split(','))
            .flat_map(|pair| {
                let mut items = pair.trim().splitn(2, '=');
                Some((items.next()?, items.next()?))
            })
        {
            match name.trim().to_lowercase().as_str() {
                "for" => realip_remote_addr.get_or_insert_with(|| unquote(val)),
                "proto" => scheme.get_or_insert_with(|| unquote(val)),
                "host" => host.get_or_insert_with(|| unquote(val)),
                "by" => {
                    continue;
                }
                _ => continue,
            };
        }

        let scheme = scheme
            .or_else(|| first_header_value(headers, &X_FORWARDED_PROTO))
            .unwrap_or("http")
            .to_owned();

        let host = host
            .or_else(|| first_header_value(headers, &X_FORWARDED_HOST))
            .or_else(|| headers.get(HOST)?.to_str().ok())
            .unwrap_or("localhost")
            .to_owned();

        let real_remote_addr = realip_remote_addr
            .or_else(|| first_header_value(headers, &X_FORWARDED_FOR))
            .map(str::to_owned);

        let path = req.uri().path().to_owned();

        let query = req.uri().query().map(str::to_owned);

        ConnectionInfo {
            scheme,
            host,
            path,
            query,
            _real_remote_addr: real_remote_addr,
        }
    }

    pub fn to_url(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);

        url.push_str(&self.path);

        url
    }

    pub fn _host(&self) -> &str {
        &self.host
    }

    pub fn _scheme(&self) -> &str {
        &self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn _real_remote_addr(&self) -> Option<&str> {
        self._real_remote_addr.as_deref()
    }
}
