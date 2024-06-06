use std::str::FromStr as _;

use axum::{
    extract::FromRef,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, head, post},
    Router,
};
use http::StatusCode;
use leptos::{get_configuration, provide_context, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use migration::{
    sea_orm::{ConnectOptions, Database, DatabaseConnection},
    Migrator, MigratorTrait,
};
use tower_http::trace::TraceLayer;
use tracing::log;

use crate::{
    fallback::file_and_error_handler, keycloak::Keycloak, middleware, routes, wasm::app::App,
    CONFIG,
};

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    #[from_ref(skip)]
    pub conn: DatabaseConnection,
    #[from_ref(skip)]
    pub keycloak: Keycloak,
    pub leptos_options: LeptosOptions,
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
    let db_url = CONFIG.database_url();
    let log_level = CONFIG.my_log();
    let filter = log::LevelFilter::from_str(log_level).unwrap_or(log::LevelFilter::Info);

    // Establish connection to the database
    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false).sqlx_logging_level(filter);
    let conn = Database::connect(opt)
        .await
        .expect("Failed to connect to the database");

    // Apply database migrations
    Migrator::up(&conn, None).await.unwrap();

    let keycloak = Keycloak::default();

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options.to_owned();

    // Build app state
    let state = AppState {
        conn,
        keycloak,
        leptos_options,
    };

    let app = app(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("Listening on: {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn app(state: AppState) -> Router {
    let health_router = Router::new()
        .route("/", get(StatusCode::OK))
        .route("/", head(StatusCode::OK));

    let auth_router = Router::new()
        .route("/logout", post(routes::logout))
        .route("/refresh", post(routes::refresh));

    let paths = generate_route_list(App);

    Router::new()
        .leptos_routes_with_context(
            &state,
            paths,
            {
                let app_state = state.to_owned();
                move || provide_context(app_state.to_owned())
            },
            App,
        )
        .fallback(file_and_error_handler)
        .layer(from_fn(middleware::headers::default))
        .layer(from_fn_with_state(
            state.to_owned(),
            middleware::authentication,
        ))
        .nest("/health", health_router)
        .nest("/auth", auth_router)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
