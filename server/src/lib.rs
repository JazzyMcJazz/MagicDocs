#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
#[cfg(feature = "ssr")]
use utils::config::Config;

#[cfg(feature = "ssr")]
pub mod database;
#[cfg(feature = "ssr")]
pub mod fallback;
#[cfg(feature = "ssr")]
pub mod keycloak;
#[cfg(feature = "ssr")]
pub mod langchain;
#[cfg(feature = "ssr")]
pub mod middleware;
#[cfg(feature = "ssr")]
pub mod models;
#[cfg(feature = "ssr")]
pub mod parsing;
#[cfg(feature = "ssr")]
pub mod responses;
#[cfg(feature = "ssr")]
pub mod routes;
#[cfg(feature = "ssr")]
pub mod server;
#[cfg(feature = "ssr")]
pub mod utils;
#[cfg(feature = "ssr")]
pub mod web_crawler;

mod markdown;
mod server_functions;
pub mod wasm;

#[cfg(feature = "ssr")]
pub static CONFIG: Lazy<Config> = Lazy::new(Config::default);

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::wasm::app::App;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
