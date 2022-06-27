use std::path::Path;

use aws_sdk_s3::model::ObjectCannedAcl;
use chrono::{SecondsFormat, Utc};
use serde_json::{json, Map, Value};

use ogcapi_drivers::{postgres::Db, s3::S3, CollectionTransactions, FeatureTransactions};
use ogcapi_types::{
    common::{media_type::JSON, Crs},
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE};

static PREFIX: &str = "mhs-upload";
static mut INIT: bool = true;

pub(crate) async fn run() -> anyhow::Result<()> {
    // Setup drivers
    let db = Db::new().await?;
    let s3 = S3::new().await;

    // List new uploads
    let resp = unsafe {
        if INIT {
            INIT = false; // requires unsafe for concurent mutability
            s3.list_objects(AWS_S3_BUCKET, Some("")).await?
        } else {
            s3.list_objects(AWS_S3_BUCKET, Some(PREFIX)).await?
        }
    };

    for object in resp.contents().unwrap_or_default() {
        // Source key
        let key = object.key().unwrap_or_default();
        if key.is_empty() || key.ends_with('/') || key.contains("/.") {
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

    // Asset id (defaults to file name)
    let p = Path::new(target);

    let asset_id = match collection_id {
        "e2e5132c-85df-417a-8706-f75068d4937e" => "meteoswiss.radar.precip",
        _ => p.file_name().unwrap_or_default().to_str().unwrap(),
    };

    // Create asset
    let mut asset = Asset::new(format!("{AWS_S3_BUCKET_BASE}/{target}"));

    asset.roles = vec!["data".to_string()];
    asset.r#type = match p.extension().unwrap_or_default().to_str() {
        Some("json") => Some(JSON.to_string()),
        Some("csv") => Some("text/csv".to_string()),
        Some("h5") => Some("application/octet-stream".to_string()),
        Some("nc") => Some("application/netcdf".to_string()),
        _ => None,
    };

    // Copy object
    s3.client
        .copy_object()
        .copy_source(format!("{AWS_S3_BUCKET}/{key}"))
        .bucket(AWS_S3_BUCKET)
        .key(target)
        .acl(ObjectCannedAcl::PublicRead)
        .send()
        .await?;

    // Update collection/item
    if collection_id == "0a62455f-c39c-4084-bd54-36ee2192d3af" {
        asset_to_collection(collection_id, asset_id, asset, db).await?;
        if key.ends_with("ch.meteoschweiz.messwerte-lufttemperatur-10min_en.json") {
            load_items_from_object(key, collection_id, db, s3).await?;
        }
    } else {
        asset_to_item(collection_id, asset_id, asset, db).await?;
    }

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

async fn asset_to_item(
    collection_id: &str,
    asset_id: &str,
    asset: Asset,
    db: &Db,
) -> anyhow::Result<()> {
    // Get item (skip key if no mapping)
    let item_id = match collection_id {
        "0a62455f-c39c-4084-bd54-36ee2192d3af" => "messwerte-lufttemperatur-10min",
        "e2e5132c-85df-417a-8706-f75068d4937e"
        | "e74c17ea-0822-44db-bef9-f37135a68245"
        | "7880287e-5d4b-4e15-b13f-846df89979a3" => "meteoswiss.radar.precip",
        _ => return Ok(()),
    };
    let mut item = db
        .read_feature(collection_id, item_id, &Crs::default())
        .await?
        .expect("existing feature");

    // Add/update datetime
    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut map = Map::new();
    map.insert("datetime".to_string(), json!(datetime));
    item.append_properties(map);

    // Add/update asset
    item.assets.insert(asset_id.to_string(), asset);

    // Write item
    db.update_feature(&item).await?;

    Ok(())
}

async fn asset_to_collection(
    collection_id: &str,
    asset_id: &str,
    asset: Asset,
    db: &Db,
) -> anyhow::Result<()> {
    // Get collection
    let mut collection = db
        .read_collection(collection_id)
        .await?
        .expect("existing collection");

    // Add/update asset
    collection.assets.insert(asset_id.to_string(), asset);

    // Write ollection
    db.update_collection(&collection).await?;

    Ok(())
}

async fn load_items_from_object(
    key: &str,
    collection_id: &str,
    db: &Db,
    s3: &S3,
) -> anyhow::Result<()> {
    // Load data
    let resp = s3.get_object(AWS_S3_BUCKET, key).await?;
    let data = resp.body.collect().await?.into_bytes();
    let geojson_str = std::str::from_utf8(&data)?;
    let geojson = geojson_str.parse::<geojson::GeoJson>()?;

    match geojson {
        geojson::GeoJson::FeatureCollection(mut fc) => {
            let mut tx = db.pool.begin().await?;

            sqlx::query(&format!(r#"TRUNCATE TABLE items."{}""#, collection_id))
                .execute(&mut tx)
                .await?;

            for (i, feature) in fc.features.iter_mut().enumerate() {
                let id = if let Some(id) = &feature.id {
                    match id {
                        geojson::feature::Id::String(id) => id.to_owned(),
                        geojson::feature::Id::Number(id) => id.to_string(),
                    }
                } else {
                    i.to_string()
                };

                if let Some(properties) = feature.properties.as_mut() {
                    properties.remove("description");
                    let datetime = properties.get("reference_ts").cloned().unwrap_or_else(|| {
                        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true))
                    });
                    properties.insert("datetime".to_string(), datetime);
                }

                sqlx::query(&format!(
                    r#"
                    INSERT INTO items."{}" (
                        id,
                        properties,
                        geom
                    ) VALUES (
                        $1,
                        $2,
                        ST_Transform(ST_SetSRID(ST_GeomFromGeoJSON($3), 2056), 4326)
                    )
                    "#,
                    collection_id
                ))
                .bind(id)
                .bind(Value::from(feature.properties.take().unwrap()))
                .bind(feature.geometry.take().unwrap().value.to_string())
                .execute(&mut tx)
                .await?;
            }

            tx.commit().await?;
        }
        _ => unimplemented!(),
    }

    Ok(())
}
