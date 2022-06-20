mod auth;
mod loader;
mod registrator;

use tower_http::auth::RequireAuthorizationLayer;

use ogcapi_services::{Config, ConfigParser, OpenAPI, Service, State};

use crate::{auth::Auth, loader::AssetLoader, registrator::AssetRegistrator};

pub static AWS_S3_BUCKET: &str = "met-oapi-poc";
pub static AWS_S3_BUCKET_BASE: &str = "http://met-oapi-poc.s3.amazonaws.com";
// static AWS_S3_BUCKET_BASE: &str = "http://localhost:9000/met-oapi-poc";

pub static OPENAPI: &[u8; 99477] = include_bytes!("../../openapi.yaml");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env
    dotenv::dotenv().ok();

    // setup tracing
    ogcapi_services::telemetry::init();

    // parse config
    let config = Config::parse();

    // application state
    let state = State::new_from(&config)
        .await
        .openapi(OpenAPI::from_slice(OPENAPI))
        .processors(vec![
            Box::new(ogcapi_services::Greeter),
            Box::new(AssetLoader),
            Box::new(AssetRegistrator),
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
