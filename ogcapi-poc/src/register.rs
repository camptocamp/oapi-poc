use std::path::Path;

use aws_sdk_s3::model::ObjectCannedAcl;
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, json};

use ogcapi_drivers::{postgres::Db, s3::S3, FeatureTransactions};
use ogcapi_types::{
    common::{media_type::JSON, Crs},
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE};

static PREFIX: &str = "mhs-upload";

pub(crate) async fn run() -> anyhow::Result<()> {
    // Setup drivers
    let db = Db::new().await?;
    let s3 = S3::new().await;

    // List new uploads
    let resp = s3.list_objects(AWS_S3_BUCKET, Some(PREFIX)).await?;

    for object in resp.contents().unwrap_or_default() {
        // Source key
        let key = object.key().unwrap_or_default();
        if key.ends_with('/') || key.is_empty() {
            continue;
        }

        // Register asset
        match ingest(key, &db, &s3).await {
            Ok(_) => {}
            Err(e) => tracing::error!("Failed registering `{key}`: {}", e),
        }
    }

    Ok(())
}

async fn ingest(key: &str, db: &Db, s3: &S3) -> anyhow::Result<()> {
    // Target key
    let target = key.trim_start_matches(PREFIX).trim_start_matches('/');

    // Collection id (uuid)
    let collection_id = target.split('/').next().unwrap_or_default();

    // Create asset
    let mut asset = Asset::new(format!("{AWS_S3_BUCKET_BASE}/{target}"));
    
    asset.roles = vec!["data".to_string()];

    let p = Path::new(target);

    asset.r#type = match p.extension().unwrap_or_default().to_str() {
        Some("json") => Some(JSON.to_string()),
        Some("csv") => Some("text/csv".to_string()),
        Some("h5") => Some("application/octet-stream".to_string()),
        Some("nc") => Some("application/netcdf".to_string()),
        _ => None,
    };

    // Asset id (defaults to file name)
    let asset_id = match collection_id {
        "e2e5132c-85df-417a-8706-f75068d4937e" => "meteoswiss.radar.precip",
        _ => p.file_name().unwrap_or_default().to_str().unwrap(),
    };

    // Get item (skip key if no mapping)
    let item_id = match collection_id {
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
    map.insert("datetime".to_string(), json!(datetime));
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
    if collection_id == "e2e5132c-85df-417a-8706-f75068d4937e" {
        let resp = s3.list_objects(AWS_S3_BUCKET, Some(collection_id)).await?;

        for object in resp.contents().unwrap_or_default() {
            let key = object.key().unwrap_or_default();
            if key != target {
                s3.delete_object(AWS_S3_BUCKET, key).await?;
            }
        }
    }

    Ok(())
}
