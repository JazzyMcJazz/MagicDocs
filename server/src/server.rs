use std::str::FromStr;

// use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, HttpResponse, HttpServer};
use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{any, get, head, post, put},
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

use crate::{keycloak::Keycloak, middleware, routes, utils::tera_testers, CONFIG};

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub tera: Tera,
    pub keycloak: Keycloak,
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

    // Initialize Tera template engine
    let Ok(mut tera) = Tera::new("templates/**/*") else {
        panic!("Failed to initialize Tera template engine");
    };
    tera.register_tester("active_project", tera_testers::active_project);
    tera.register_tester("active_document", tera_testers::active_document);
    tera.register_tester("permitted", tera_testers::permitted);

    let keycloak = Keycloak::new().await.unwrap();

    // Build app state
    let state = AppState {
        conn,
        tera,
        keycloak,
    };

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

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
        .route("/:project_id", get(routes::projects::redirect_to_latest))
        .route("/:project_id/v/:version", get(routes::projects::detail))
        .route("/:project_id/v/:version/chat", post(routes::chat))
        .route(
            "/:project_id/v/:version/finalize",
            post(routes::projects::finalize),
        )
        .route(
            "/:project_id/v/:version/documents",
            post(routes::document::create),
        )
        .route(
            "/:project_id/v/:version/documents/editor",
            get(routes::document::new),
        )
        .route(
            "/:project_id/v/:version/documents/crawler",
            get(routes::document::new),
        )
        .route(
            "/:project_id/v/:version/documents/crawler",
            post(routes::document::crawler),
        )
        .route(
            "/:project_id/v/:version/documents/:doc_id",
            any(routes::document::detail).patch(routes::document::patch),
        )
        .route(
            "/:project_id/v/:version/documents/:doc_id/edit",
            get(routes::document::editor),
        )
        .layer(from_fn_with_state(
            (true, state.to_owned()),
            middleware::authorization,
        ));

    let admin_router = Router::new()
        .route("/", get(routes::admin::dashboard))
        .route("/users", get(routes::admin::users))
        .route("/users/:user_id", get(routes::admin::user_details))
        .route("/users/:user_id", put(routes::admin::update_user))
        .route("/roles", get(routes::admin::roles))
        .route("/roles", post(routes::admin::create_role))
        .route("/roles/:role_name", get(routes::admin::role_details))
        .route(
            "/roles/:role_name/permissions",
            post(routes::admin::update_role_permissions),
        )
        .layer(from_fn_with_state(
            (false, state.to_owned()),
            middleware::authorization,
        ));

    let main_router = Router::new()
        .route("/", get(routes::index))
        .route("/logout", post(routes::logout))
        .route("/flush", post(routes::refresh))
        .nest("/projects", project_router)
        .nest("/admin", admin_router)
        .layer(from_fn_with_state(
            state.to_owned(),
            middleware::context_builder,
        ))
        .layer(from_fn_with_state(
            state.to_owned(),
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
