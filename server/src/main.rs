use dotenvy::dotenv;
use env_logger::Env;
use once_cell::sync::Lazy;
use utils::config::Config;

mod database;
mod keycloak;
mod langchain;
mod middleware;
mod models;
mod parsing;
mod responses;
mod routes;
mod server;
mod utils;
mod web_crawler;

static CONFIG: Lazy<Config> = Lazy::new(Config::default);

fn main() {
    dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("info")
            .default_write_style_or("always".to_string()),
    );

    let _ = server::run();
}
