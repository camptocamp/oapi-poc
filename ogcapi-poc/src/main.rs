mod auth;
mod loader;

use tower_http::auth::RequireAuthorizationLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::Auth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup env
    dotenv::dotenv().ok();

    // setup tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // parse config
    let config = ogcapi_services::parse_config();

    // setup database connection pool & run any pending migrations
    let db = ogcapi_drivers::postgres::Db::setup(&config.database_url).await?;

    // build application
    let router = ogcapi_services::app(db).await;

    // add custom basic auth
    let router = router.route_layer(RequireAuthorizationLayer::custom(Auth));

    // run our app with hyper
    let address = &format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("listening on {}", address);

    axum::Server::bind(address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(ogcapi_services::shutdown_signal())
        .await
        .unwrap();

    Ok(())
}
