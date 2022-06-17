use std::path::Path;

use anyhow::Context;
use aws_sdk_s3::model::ObjectCannedAcl;
use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use schemars::{gen::SchemaSettings, JsonSchema};
use serde::Deserialize;
use serde_json::Map;
use url::Url;

use ogcapi_services::{Error, Processor, Result, State};
use ogcapi_types::{
    common::{media_type::JSON, Crs},
    processes::{Execute, Process},
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE};

/// STAC Asset registrator
pub(crate) struct AssetRegistrator;

/// Input schema
#[derive(Deserialize, Debug, JsonSchema)]
struct Inputs {
    /// S3 prefix to ceck for new files
    #[serde(default)]
    prefix: String,
}

/// Output schema
#[derive(JsonSchema)]
struct Outputs(String);

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

    async fn execute(&self, execute: Execute, state: &State, _url: &Url) -> Result<Response> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: Inputs = serde_json::from_value(value)
            .map_err(|e| Error::Exception(StatusCode::BAD_REQUEST, e.to_string()))?;

        let datetime = Utc::now();

        // Register assets
        let resp = state
            .s3
            .client
            .list_objects()
            .bucket(AWS_S3_BUCKET)
            .prefix(&inputs.prefix)
            .send()
            .await
            .context("List objects")?;

        for object in resp.contents().unwrap_or_default() {
            // Copy object
            let key = object.key().unwrap_or_default();
            tracing::debug!("key: {}", key);

            if let Some(target) = match key {
                "ch.meteoschweiz.messwerte-lufttemperatur-10min/messwerte-lufttemperatur-10min/ch.meteoschweiz.messwerte-lufttemperatur-10min.json" => Some("ch.meteoschweiz.messwerte-lufttemperatur-10min/messwerte-lufttemperatur-10min/test2.json"),
                "ch.meteoschweiz.messwerte-lufttemperatur-10min_en.json" => Some("ch.meteoschweiz.messwerte-lufttemperatur-10min/messwerte-lufttemperatur-10min/test.json"),
                "meteoswiss.radar.precip.202206170542.h5" => Some("ch.meteoschweiz.messwerte-lufttemperatur-10min/messwerte-lufttemperatur-10min/meteoswiss.radar.precip.202206170542.h5"),
                _ => None
            } {
                tracing::debug!("target: {}", target);
                state
                    .s3
                    .client
                    .copy_object()
                    .copy_source(format!("{AWS_S3_BUCKET}/{key}"))
                    .bucket(AWS_S3_BUCKET)
                    .key(target)
                    .acl(ObjectCannedAcl::PublicRead)
                    .send()
                    .await
                    .context("Copy object")?;

                let p = Path::new(target);

                // Get item
                let mut parts = p.iter();
                let collection = parts.next().unwrap().to_str().unwrap();
                let id = parts
                    .next()
                    .and_then(|s| s.to_str())
                    .unwrap_or("messwerte-lufttemperatur-10min");

                let mut item = state
                    .drivers
                    .features
                    .read_feature(collection, id, &Crs::default())
                    .await?;

                // Add/update datetime
                let mut map = Map::new();
                map.insert(
                    "datetime".to_string(),
                    serde_json::to_value(datetime).unwrap(),
                );
                item.append_properties(map);

                // Add/update asset
                let mut asset = Asset::new(format!("{AWS_S3_BUCKET_BASE}/{target}"));
                asset.roles = vec!["data".to_string()];
                asset.r#type = match p.extension().unwrap().to_str().unwrap_or_default() {
                    "json" => Some(JSON.to_string()),
                    _ => None,
                };

                let asset_id = p.file_stem().unwrap().to_str().unwrap();
                item.assets.insert(asset_id.to_string(), asset);

                // Write item
                state.drivers.features.update_feature(&item).await?;
                println!("{}", key);
            }
        }

        Ok(StatusCode::OK.into_response())
    }
}
