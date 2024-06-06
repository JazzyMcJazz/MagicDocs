use dotenvy::dotenv;
use env_logger::Env;
use magicdocs::server;

#[cfg(feature = "ssr")]
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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}
