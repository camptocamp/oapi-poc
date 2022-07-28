mod auth;
mod initialization;
mod loader;
mod observation;
mod proj;
mod register;

use axum::{handler::Handler, response::IntoResponse};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::auth::RequireAuthorizationLayer;

use ogcapi_services::{Config, ConfigParser, Error, OpenAPI, Service, State};
use ogcapi_types::common::LandingPage;

use crate::{auth::Auth, loader::AssetLoader};

pub static ROOT: &str = "https://poc.meteoschweiz-poc.swisstopo.cloud/root";
pub static AWS_S3_BUCKET: &str = "met-oapi-poc";
pub static AWS_S3_BUCKET_BASE: &str = "https://s3.meteoschweiz-poc.swisstopo.cloud";
// pub static AWS_S3_BUCKET_BASE: &str = "http://localhost:9000/met-oapi-poc";

pub static OPENAPI: &[u8; 63483] = include_bytes!("../../openapi.yaml");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env
    dotenv::dotenv().ok();

    // setup tracing
    ogcapi_services::telemetry::init();

    // parse config
    let config = Config::parse();

    // initialize
    if std::env::var("INITIALIZE").unwrap_or_else(|_| "false".to_string()) == "true" {
        tracing::info!("Initializing ...");
        initialization::init(&config.database_url).await?;
    }

    // landing page
    let root = LandingPage::new("root")
        .title("PoC MeteoSchweiz")
        .description(include_str!("../../README.md"));

    // application state
    let state = State::new_from(&config)
        .await
        .root(root)
        .openapi(OpenAPI::from_slice(OPENAPI))
        .processors(vec![
            Box::new(ogcapi_services::Greeter),
            Box::new(AssetLoader),
        ]);

    // create service
    let mut service = Service::new_with(&config, state).await;

    // add custom basic auth
    service.router = axum::Router::new()
        .nest(
            "/root/",
            service
                .router
                .route_layer(RequireAuthorizationLayer::custom(Auth)),
        )
        .fallback(handler_404.into_service());

    // cron job to register assets
    let sched = JobScheduler::new()?;
    sched.add(
        Job::new_async("30 1/1 * * * *", |_uuid, _l| {
            Box::pin(async move {
                tracing::info!("register assets");
                register::run("mhs-upload").await.unwrap();
            })
        })
        .unwrap(),
    )?;
    sched.start()?;

    // run service with hyper
    service.serve().await;

    Ok(())
}

pub async fn handler_404() -> impl IntoResponse {
    Error::NotFound
}
