use std::{env, str::FromStr};

// use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, HttpResponse, HttpServer};
use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, head, post},
    Router,
};
use http::StatusCode;
use migration::{
    sea_orm::{ConnectOptions, Database, DatabaseConnection},
    Migrator, MigratorTrait,
};
use tera::Tera;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::log;

use crate::{keycloak::Keycloak, middleware, routes, utils::tera_testers};

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub tera: Tera,
    pub keycloak: Keycloak,
}

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL in .env");
    let log_level = env::var("MY_LOG").unwrap_or_else(|_| "info".to_string());
    let filter = log::LevelFilter::from_str(&log_level).unwrap_or(log::LevelFilter::Info);

    // Establish connection to the database
    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false).sqlx_logging_level(filter);
    let conn = Database::connect(opt)
        .await
        .expect("Failed to connect to the database");

    // Apply database migrations
    Migrator::up(&conn, None).await.unwrap();

    // Initialize Tera template engine
    let Ok(mut tera) = Tera::new("templates/**/*") else {
        panic!("Failed to initialize Tera template engine");
    };
    tera.register_tester("active_project", tera_testers::active_project);
    tera.register_tester("active_document", tera_testers::active_document);

    let keycloak = Keycloak::new().await.unwrap();

    // Build app state
    let state = AppState {
        conn,
        tera,
        keycloak,
    };

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn app(state: AppState) -> Router {
    let static_files_router = Router::new()
        .nest_service("/", ServeDir::new("static"))
        .layer(from_fn(middleware::headers::cache_control));

    let health_router = Router::new()
        .route("/", get(StatusCode::OK))
        .route("/", head(StatusCode::OK));

    let browser_sync_router = Router::new().route("/", get(routes::browser_sync::sse));

    let project_router = Router::new()
        .route("/", post(routes::projects::create))
        .route("/new", get(routes::projects::new))
        .route("/:id", get(routes::projects::redirect_to_latest))
        .route("/:id/v/:version", get(routes::projects::detail))
        .route("/:id/v/:version/finalize", post(routes::projects::finalize))
        .route("/:id/v/:version/documents", post(routes::document::create))
        .route(
            "/:id/v/:version/documents/editor",
            get(routes::document::new),
        )
        .route(
            "/:id/v/:version/documents/crawler",
            get(routes::document::new),
        )
        .route(
            "/:id/v/:version/documents/crawler",
            post(routes::document::crawler),
        )
        .route(
            "/:id/v/:version/documents/:doc_id",
            get(routes::document::detail),
        )
        .layer(from_fn_with_state(true, middleware::authorization));

    let admin_router = Router::new()
        .route("/", get(routes::admin::dashboard))
        .route("/users", get(routes::admin::users))
        .route("/roles", get(routes::admin::roles))
        .route("/permissions", get(routes::admin::permissions));

    let main_router = Router::new()
        .route("/", get(routes::index))
        .route("/logout", post(routes::logout))
        .route("/flush", post(routes::refresh))
        .nest("/projects", project_router)
        .nest("/admin", admin_router)
        .layer(from_fn_with_state(
            state.clone(),
            middleware::context_builder,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::authentication,
        ));

    Router::new()
        .nest("/health", health_router)
        .nest("/static", static_files_router)
        .nest("/browser-sync", browser_sync_router)
        .nest("/", main_router)
        .layer(from_fn(middleware::headers::default))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
