mod auth;
mod loader;
mod register;

use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::auth::RequireAuthorizationLayer;

use ogcapi_services::{Config, ConfigParser, OpenAPI, Service, State};
use ogcapi_types::common::{link_rel::CHILD, LandingPage, Link};

use crate::{auth::Auth, loader::AssetLoader};

pub static AWS_S3_BUCKET: &str = "met-oapi-poc";
pub static AWS_S3_BUCKET_BASE: &str = "http://met-oapi-poc.s3.amazonaws.com";
// pub static AWS_S3_BUCKET_BASE: &str = "http://localhost:9000/met-oapi-poc";

pub static OPENAPI: &[u8; 99477] = include_bytes!("../../openapi.yaml");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env
    dotenv::dotenv().ok();

    // setup tracing
    ogcapi_services::telemetry::init();

    // parse config
    let config = Config::parse();

    // landing page
    let root = LandingPage::new("root")
        .title("PoC MeteoSchweiz")
        .description(include_str!("../../README.md"));
    // .links(vec![Link::new("collections/test-child", CHILD)]);

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
    service.router = service
        .router
        .route_layer(RequireAuthorizationLayer::custom(Auth));

    // cron job to register assets
    let sched = JobScheduler::new()?;
    sched.add(
        Job::new_async("0 1/1 * * * *", |_uuid, _l| {
            Box::pin(async {
                tracing::info!("register assets");
                register::run().await.unwrap();
            })
        })
        .unwrap(),
    )?;
    sched.start()?;

    // run service with hyper
    service.serve().await;

    Ok(())
}
