use anyhow::Result;
use axum_extra::extract::cookie::Cookie;
use time::OffsetDateTime;

use crate::keycloak::TokenResponse;

use super::config::Config;

pub fn from_token_response(
    token_response: TokenResponse,
) -> Result<(Cookie<'static>, Cookie<'static>, Cookie<'static>)> {
    let config = Config::default();
    let is_test = config.rust_env() == "test";

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let expires = OffsetDateTime::from_unix_timestamp(now + token_response.expires_in() - 10)?; // 10 seconds before the token expires
    let rf_expires = OffsetDateTime::from_unix_timestamp(now + 60 * 60 * 24 * 180)?; // 180 days

    let id_cookie = Cookie::build(("id", token_response.id_token().to_owned()))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .build();
    let access_cookie = Cookie::build(("ac", token_response.access_token().to_owned()))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .build();
    let refresh_cookie = Cookie::build(("rf", token_response.refresh_token().to_owned()))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(rf_expires)
        .build();

    Ok((id_cookie, access_cookie, refresh_cookie))
}

pub fn expire() -> (Cookie<'static>, Cookie<'static>, Cookie<'static>) {
    let config = Config::default();
    let is_test = config.rust_env() == "test";

    let expires = OffsetDateTime::UNIX_EPOCH;

    let cookie1 = Cookie::build(("id", ""))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .build();
    let cookie2 = Cookie::build(("ac", ""))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .build();
    let cookie3 = Cookie::build(("rf", ""))
        .path("/")
        .secure(!is_test)
        .http_only(true)
        .expires(expires)
        .build();

    (cookie1, cookie2, cookie3)
}
