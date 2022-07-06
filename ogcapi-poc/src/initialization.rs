use std::ffi::OsStr;

use include_dir::{include_dir, Dir};
use reqwest::Url;
use sqlx::{migrate::MigrateDatabase, ConnectOptions, PgPool};

use ogcapi_drivers::{postgres::Db, CollectionTransactions, FeatureTransactions};
use ogcapi_types::{common::Collection, features::Feature};

static COLLECTIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../collections");

pub(crate) async fn init(database_url: &Url) -> anyhow::Result<()> {
    // drop database
    sqlx::Postgres::drop_database(database_url.as_str()).await?;

    // setup database
    let mut db = Db::setup(database_url).await?;

    // disable query logging
    let mut options = db.pool.connect_options().to_owned();
    options.disable_statement_logging();
    let pool = PgPool::connect_with(options).await.unwrap();
    db = Db { pool };

    // load resources
    for entry in COLLECTIONS_DIR.find("*.json").unwrap() {
        if entry
            .path()
            .components()
            .any(|c| c.as_os_str() == OsStr::new("items"))
        {
            continue;
        }

        let file = entry.as_file().unwrap().contents();
        let collection: Collection = serde_json::from_slice(file)?;
        let collection_id = &collection.id;
        tracing::info!("Initializing `{}`", collection_id);
        db.create_collection(&collection).await?;

        for entry in COLLECTIONS_DIR
            .find(&format!("{collection_id}/items/*.json"))
            .unwrap()
        {
            let file = entry.as_file().unwrap().contents();
            let feature: Feature = serde_json::from_slice(file)?;
            db.create_feature(&feature).await?;
        }
    }

    // register/load assets
    tracing::info!("run full registration");
    crate::register::run("").await.unwrap();

    Ok(())
}
