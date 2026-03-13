mod config;
mod controllers;
mod repositories;
mod services;
mod core;
mod models;
mod responses;
mod dto;

use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::Router;
use anyhow::Result;
use axum::http::{HeaderName, Method};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::LatencyUnit;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tracing_subscriber::EnvFilter;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};

use crate::controllers::auth::auth_routes;
use crate::controllers::storage::storage_routes;
use crate::core::database::connection::{create_connection_db, create_connection_redis};

use crate::core::state::AppState;


#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info".to_string()));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init();
    // state params
    let cfg = config::AppConfig::from_env();
    let db_connection = create_connection_db(&cfg).await?;
    let redis_connection = create_connection_redis(&cfg).await?;

    let app_state = AppState { cfg: Arc::new(cfg.clone()), db: db_connection, redis: Arc::new(redis_connection), storage_root: std::path::PathBuf::from(cfg.root_dir.as_str()) };
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT, HeaderName::from_static("x-requested-with")])
        .allow_credentials(true);

    let app = Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
        .merge(auth_routes())
        .merge(storage_routes())
        .layer(cors)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(app_state);

    let ip: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(ip, 9099);
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::auth::login,
        crate::controllers::auth::send_verification_code,
        crate::controllers::auth::confirm_registration,
        crate::controllers::storage::upload_file,
        crate::controllers::storage::remove_file,
        crate::controllers::storage::create_directory,
        crate::controllers::storage::remove_directory,
        crate::controllers::storage::rename_directory
    )
)]
pub struct ApiDoc;