mod auth;
mod loader;

use tower_http::auth::RequireAuthorizationLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ogcapi_services::{Config, ConfigParser, OpenAPI, Service, State};

use crate::{auth::Auth, loader::AssetLoader};

pub static OPENAPI: &[u8; 99477] = include_bytes!("../../openapi.yaml");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env
    dotenv::dotenv().ok();

    // setup tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // parse config
    let config = Config::parse();

    // application state
    let state = State::new_from(&config)
        .await
        .openapi(OpenAPI::from_slice(OPENAPI))
        .processors(vec![
            Box::new(ogcapi_services::Greeter),
            Box::new(AssetLoader),
        ]);

    // create service
    let mut service = Service::new_with(&config, state).await;

    // add custom basic auth
    service.router = service
        .router
        .route_layer(RequireAuthorizationLayer::custom(Auth));

    // run service with hyper
    service.serve().await;

    Ok(())
}
