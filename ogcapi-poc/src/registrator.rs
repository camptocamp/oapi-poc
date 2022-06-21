use std::path::Path;

use anyhow::Context;
use aws_sdk_s3::model::ObjectCannedAcl;
use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{SecondsFormat, Utc};
use ogcapi_drivers::{postgres::Db, s3::S3, FeatureTransactions};
use schemars::{gen::SchemaSettings, JsonSchema};
use serde::Deserialize;
use serde_json::Map;
use url::Url;

use ogcapi_services::{Processor, Result, State};
use ogcapi_types::{
    common::{media_type::JSON, Crs},
    processes::{Execute, Process},
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE};

static PREFIX: &str = "mhs-upload";

/// STAC Asset registrator
pub(crate) struct AssetRegistrator;

/// Input schema
#[derive(Deserialize, Debug, JsonSchema)]
struct Inputs;

/// Output schema
#[derive(JsonSchema)]
struct Outputs;

#[async_trait]
impl Processor for AssetRegistrator {
    fn id(&self) -> String {
        "register-assets".to_string()
    }
    fn process(&self) -> Process {
        // Config schema generation
        let settings = SchemaSettings::default().with(|s| {
            s.option_nullable = false;
            s.option_add_null_type = false;
            s.inline_subschemas = true;
        });
        let gen = settings.into_generator();

        Process::new(
            self.id(),
            "0.1.0",
            &serde_json::to_value(&gen.clone().into_root_schema_for::<Inputs>().schema).unwrap(),
            &serde_json::to_value(&gen.into_root_schema_for::<Outputs>().schema).unwrap(),
        )
    }

    async fn execute(&self, _execute: Execute, state: &State, _url: &Url) -> Result<Response> {
        // List new uploads
        let resp = state
            .s3
            .client
            .list_objects()
            .bucket(AWS_S3_BUCKET)
            .prefix(PREFIX)
            .send()
            .await
            .context("List objects")?;

        for object in resp.contents().unwrap_or_default() {
            // Source key
            let key = object.key().unwrap_or_default();
            tracing::debug!("key: {}", key);

            // Register asset
            match register(key, &state.db, &state.s3).await {
                Ok(_) => tracing::info!("finish registering `{key}`"),
                Err(e) => tracing::error!("error registering`{key}`: {}", e.to_string()),
            }
        }

        Ok(StatusCode::OK.into_response())
    }
}

async fn register(key: &str, db: &Db, s3: &S3) -> anyhow::Result<()> {
    // Target key
    let target = key.trim_start_matches(PREFIX).trim_start_matches('/');
    tracing::debug!("target: {}", target);

    // Collection id (uuid)
    let collection_id = &target.split('/').next().unwrap_or_default();

    // Create asset
    let mut asset = Asset::new(format!("{AWS_S3_BUCKET_BASE}/{target}"));
    asset.roles = vec!["data".to_string()];

    let p = Path::new(target);
    asset.r#type = match p
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "json" => Some(JSON.to_string()),
        "csv" => Some("text/csv".to_string()),
        "h5" => Some("application/octet-stream".to_string()),
        "nc" => Some("application/netcdf".to_string()),
        _ => None,
    };

    // Asset id (defaults to file name)
    let asset_id = match *collection_id {
        "e2e5132c-85df-417a-8706-f75068d4937e" => "meteoswiss.radar.precip",
        _ => p
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
    };

    // Get item (skip key if no item id can be mapped)
    let item_id = match *collection_id {
        "0a62455f-c39c-4084-bd54-36ee2192d3af" => "messwerte-lufttemperatur-10min",
        "e2e5132c-85df-417a-8706-f75068d4937e" => "meteoswiss.radar.precip",
        _ => return Ok(()),
    };
    let mut item = db
        .read_feature(collection_id, item_id, &Crs::default())
        .await?;

    // Add/update datetime
    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut map = Map::new();
    map.insert("datetime".to_string(), serde_json::to_value(datetime)?);
    item.append_properties(map);

    // Add/update asset
    item.assets.insert(asset_id.to_string(), asset);

    // Copy object
    s3.client
        .copy_object()
        .copy_source(format!("{AWS_S3_BUCKET}/{key}"))
        .bucket(AWS_S3_BUCKET)
        .key(target)
        .acl(ObjectCannedAcl::PublicRead)
        .send()
        .await?;

    // Write item
    db.update_feature(&item).await?;

    // Cleanup
    s3.delete_object(AWS_S3_BUCKET, key).await?;

    Ok(())
}
