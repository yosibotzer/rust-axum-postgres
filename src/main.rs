mod controller;
mod model;
mod service;

use std::env;

use axum::{http::StatusCode, routing::get, Router};
use model::service_config::ServiceConfig;
use model::service_state::ServiceState;
use strum_macros::EnumString;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::Postgres;

use crate::controller::book_controller::get_book_routes;

#[derive(Debug, EnumString, strum_macros::Display)]
enum RunMode {
    Dev,
    Prod,
    Test,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let run_mode_str = env::var("RUN_MODE").unwrap_or_else(|_| RunMode::Dev.to_string());

    let run_mode: RunMode = run_mode_str.parse()?;

    set_tracing(&run_mode)?;

    info!("run mode: {:?}", run_mode);
    
    let service_config = ServiceConfig::new(&run_mode)?;
    let pg_pool = get_pg_pool(&service_config).await?;

    sqlx::migrate!("./migrations").run(&pg_pool).await?;
    
    let service_state = ServiceState { 
        pg_pool
     };

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let app = Router::new()
        .route("/status", get(status))
        .merge(get_book_routes(service_state))
        .layer(CompressionLayer::new());

    info!("starting server {:?}", listener);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn status() -> (StatusCode, String) {
    (StatusCode::OK, "Everything is OK".to_string())
}

fn set_tracing(run_mode: &RunMode) -> Result<(), Box<dyn std::error::Error>> {
    
    match run_mode {
        
        RunMode::Dev | RunMode::Test => {
            
            let env_filter = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(LevelFilter::INFO.to_string()))?;
            
            let simple_collector = tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(env_filter);
            
            tracing::subscriber::set_global_default(simple_collector)?;
            
            Ok(())
        }
        RunMode::Prod => {
            
            let env_filter = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(LevelFilter::WARN.to_string()))?;
            
            let logger = tracing_logstash::Layer::default().event_format(
                tracing_logstash::logstash::LogstashFormat::default()
                    .with_constants(vec![("service.name", "rust-axum-postgres".to_owned())]),
            );

            let collector = Registry::default()
                .with(logger)
                .with(env_filter);
            
            tracing::subscriber::set_global_default(collector)?;
            
            Ok(())
        }
    }
}

async fn get_pg_pool(service_config: &ServiceConfig,) -> Result<sqlx::Pool<Postgres>, Box<dyn std::error::Error>> {
    
    info!("connecting to postgres");

    let pg_pool: sqlx::Pool<Postgres> = PgPoolOptions::new()
        .max_connections(service_config.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(service_config.acquire_timeout_sec,))
        .connect(service_config.url.as_str())
        .await?;

    info!("pg pool: {:?}", pg_pool);

    Ok(pg_pool) 
}
