use std::{env, str::FromStr};

use actix_files as fs;
use actix_web::{dev::ServiceRequest, middleware::Logger, web, App, HttpResponse, HttpServer};
use migration::{
    sea_orm::{ConnectOptions, Database, DatabaseConnection},
    Migrator, MigratorTrait,
};
use tera::Tera;
use tracing::log;

use crate::{keycloak::Keycloak, middleware, routes};

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub tera: Tera,
    pub keycloak: Keycloak,
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL in .env");
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "prod".to_string());

    // Establish connection to the database
    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false).sqlx_logging_level(
        log::LevelFilter::from_str(&log_level).unwrap_or(log::LevelFilter::Info),
    );
    let conn = Database::connect(opt)
        .await
        .expect("Failed to connect to the database");

    // Apply database migrations
    Migrator::up(&conn, None).await.unwrap();

    // Initialize Tera template engine
    let Ok(tera) = Tera::new("templates/**/*") else {
        panic!("Failed to initialize Tera template engine");
    };

    let keycloak = Keycloak::new().await.unwrap();

    // Build app state
    let state = AppState {
        conn,
        tera,
        keycloak,
    };

    // Start the HTTP server
    let mut server = HttpServer::new(move || {
        let cache_control = if rust_env == "dev" {
            "no-store"
        } else {
            "max-age=600"
        };

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .service(
                web::scope("/health")
                    .route("", web::head().to(HttpResponse::Ok))
                    .route("", web::get().to(HttpResponse::Ok)),
            )
            .service(
                web::scope("/static")
                    .wrap(
                        actix_web::middleware::DefaultHeaders::new()
                            .add(("Cache-Control", cache_control)),
                    )
                    .service(
                        fs::Files::new("", "static")
                            .index_file("invalid")
                            .default_handler(|req: ServiceRequest| async {
                                Ok(req.into_response(HttpResponse::NotFound()))
                            }),
                    ),
            )
            .service(
                web::scope("")
                    .wrap(middleware::ContextSetter) // 2
                    .wrap(middleware::Authentication) // 1
                    .configure(init),
            )
    });

    server = server.bind("0.0.0.0:3000")?;
    server.run().await?;

    Ok(())
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(routes::index));
    cfg.route("/logout", web::get().to(routes::logout));
    cfg.route("/flush", web::post().to(routes::refresh));

    cfg.service(
        web::scope("/projects")
            .wrap(middleware::Authorization { admin: true })
            .route("/new", web::get().to(routes::projects::new)),
    );
}
