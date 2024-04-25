use actix_web::cookie::{self, time::OffsetDateTime, Cookie};

use crate::keycloak::TokenResponse;

pub fn from_token_response(
    token_response: &'_ TokenResponse,
) -> Result<(Cookie<'_>, Cookie<'_>, Cookie<'_>), Box<dyn std::error::Error>> {
    let is_test = std::env::var("RUST_ENV").unwrap_or_else(|_| "".to_string()) == "test";

    let now = cookie::time::OffsetDateTime::now_utc().unix_timestamp();
    let expires =
        cookie::time::OffsetDateTime::from_unix_timestamp(now + token_response.expires_in() - 10)?; // 10 seconds before the token expires
    let rf_expires = cookie::time::OffsetDateTime::from_unix_timestamp(now + 60 * 60 * 24 * 180)?; // 180 days

    let id_cookie = Cookie::build("id", token_response.id_token())
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .finish();

    let access_cookie = Cookie::build("ac", token_response.access_token())
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .finish();

    let refresh_cookie = Cookie::build("rf", token_response.refresh_token())
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(rf_expires)
        .finish();

    Ok((id_cookie, access_cookie, refresh_cookie))
}

pub fn expire() -> (Cookie<'static>, Cookie<'static>, Cookie<'static>) {
    let expires = OffsetDateTime::UNIX_EPOCH;

    let cookie1 = Cookie::build("id", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();
    let cookie2 = Cookie::build("ac", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();
    let cookie3 = Cookie::build("rf", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .expires(expires)
        .finish();

    (cookie1, cookie2, cookie3)
}
