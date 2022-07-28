use std::path::Path;

use anyhow::bail;
use aws_sdk_s3::model::ObjectCannedAcl;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, SecondsFormat, Utc};
use geo::Transform;
use serde_json::json;
use tokio_stream::StreamExt;

use ogcapi_drivers::{postgres::Db, s3::S3, CollectionTransactions, FeatureTransactions};
use ogcapi_types::{
    common::{
        media_type::{GEO_JSON, JSON},
        Crs,
    },
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE, ROOT};

pub(crate) async fn run(prefix: &str) -> anyhow::Result<()> {
    let now = Utc::now();
    // Setup drivers
    let db = Db::new().await?;
    let s3 = S3::new().await;

    // List new uploads
    let mut paginator = s3
        .client
        .list_objects_v2()
        .bucket(AWS_S3_BUCKET)
        .prefix(prefix)
        .into_paginator()
        .send();

    while let Some(resp) = paginator.next().await {
        // Register assets
        for object in resp.unwrap().contents().unwrap_or_default() {
            // Last modified
            let mut datetime = object.last_modified.unwrap().to_chrono_utc();
            let age = now - datetime;

            // Source key
            let source = object.key().unwrap_or_default();
            if source.is_empty()
                || source.ends_with('/')
                || source.contains("/.")
                // || (prefix.is_empty() && source.starts_with("mhs-upload"))
                || (prefix.is_empty() && source.starts_with("a6296aa9-d183-45c3-90fc-f03ec7d637be"))
                || (!prefix.is_empty() && age.num_seconds() < 10)
                || (!prefix.is_empty() && age.num_seconds() >= 70 && age.num_minutes() % 5 != 0)
            {
                continue;
            }

            // Target key
            let mut target = source
                .trim_start_matches("mhs-upload")
                .trim_start_matches('/')
                .to_owned();

            // Collection id (uuid)
            let collection_id = target.split('/').next().unwrap_or_default().to_owned();

            // Get datetime from filename for alerts
            if collection_id == "35ff8133-364a-47eb-a145-0d641b706bff" {
                datetime = DateTime::parse_from_str(
                    &format!("{}+0000", target.split('.').nth(2).unwrap()),
                    "%Y%m%d%H%M%z",
                )?
                .into();
            }

            // Model perculiarities
            if collection_id == "a6296aa9-d183-45c3-90fc-f03ec7d637be" {
                // cut initime form target
                let start = target.find("initime").unwrap();
                let end = start + 19; // xxx_initime_2022062300_xxx

                datetime = DateTime::parse_from_str(
                    &format!("{}00+0000", &target[start..end].split('_').nth(1).unwrap()),
                    "%Y%m%d%H%M%z",
                )?
                .into();

                target = format!("{}{}", &target[..start], &target[end..]);
            }

            // Asset id (defaults to file name)
            let asset_id = Path::new(&target)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap();

            // Create asset
            let mut asset = Asset::new(format!("{AWS_S3_BUCKET_BASE}/{target}"));
            asset.roles = vec!["data".to_string()];
            asset.r#type = match target.split('.').last() {
                Some("json") => Some(JSON.to_string()),
                Some("csv") => Some("text/csv".to_string()),
                Some("h5") => Some("application/x-hdf5".to_string()),
                Some("nc") => Some("application/netcdf".to_string()),
                Some("tiff") => Some("image/tiff".to_string()),
                Some("cap") => Some("text/xml".to_string()),
                Some("zip") => Some("application/zip".to_string()),
                Some("grib2") => Some("application/wmo-grib2".to_string()),
                _ => None,
            };

            // Update collection/item
            let result = match collection_id.as_str() {
                "0a62455f-c39c-4084-bd54-36ee2192d3af" | "ad2b1452-9f3c-4137-9822-9758298bc025" => {
                    if source != target {
                        copy_object(source, &target, &s3).await.unwrap();
                    }

                    if source.ends_with("ch.meteoschweiz.messwerte-lufttemperatur-10min_en.json")
                        || source.ends_with("observations-hourly.csv")
                    {
                        load_items_from_object(source, &collection_id, &db, &s3)
                            .await
                            .unwrap();
                    }
                    asset_to_collection(&collection_id, asset_id, asset, &db).await
                }
                c => {
                    // Get item (skip key if no mapping)
                    let item_id = match c {
                        "e2e5132c-85df-417a-8706-f75068d4937e"
                        | "e74c17ea-0822-44db-bef9-f37135a68245"
                        | "7880287e-5d4b-4e15-b13f-846df89979a3" => "meteoswiss.radar.precip",
                        "ed6a30c9-672e-4d8f-95e4-8c5bef8ab417" => {
                            "klimanormwerte.temperatur.1961-1990"
                        }
                        "b46a8f8d-bc48-41d3-b20a-de61d0763318" => {
                            asset_id.split('_').nth(1).unwrap()
                        }
                        "4ccc5153-cc27-47b8-abee-9d6e12e19701" => {
                            &asset_id.split('_').last().unwrap()[..8]
                        }
                        "35ff8133-364a-47eb-a145-0d641b706bff" => asset_id.trim_end_matches(".cap"),
                        "a6296aa9-d183-45c3-90fc-f03ec7d637be" => {
                            asset_id.trim_end_matches(".grib2")
                        }
                        _ => {
                            tracing::warn!("no mapping for collection `{collection_id}`");
                            continue;
                        }
                    };

                    if source != target {
                        copy_object(source, &target, &s3).await.unwrap();
                    }

                    asset_to_item(item_id, &collection_id, asset_id, asset, &datetime, &db).await
                }
            };

            // Cleanup
            match result {
                Ok(_) => {
                    if !prefix.is_empty() {
                        s3.delete_object(AWS_S3_BUCKET, source).await?;
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to load asset: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn asset_to_item(
    item_id: &str,
    collection_id: &str,
    asset_id: &str,
    asset: Asset,
    datetime: &DateTime<Utc>,
    db: &Db,
) -> anyhow::Result<()> {
    let mut item = match db
        .read_feature(collection_id, item_id, &Crs::default())
        .await?
    {
        Some(feature) => feature,
        None => {
            if collection_id == "4ccc5153-cc27-47b8-abee-9d6e12e19701"
                || collection_id == "35ff8133-364a-47eb-a145-0d641b706bff"
            {
                let feature = serde_json::from_value(json!(
                    {
                        "id": item_id,
                        "collection": collection_id,
                        "geometry": {
                            "type": "Polygon",
                            "coordinates": [[
                                [5.96, 45.82],
                                [10.49,45.82],
                                [10.49,47.81],
                                [5.96,47.81],
                                [5.96,45.82]
                            ]]
                        },
                        "bbox": [5.96, 45.82, 10.49, 47.81],
                        "properties": {}
                    }
                ))
                .unwrap();

                db.create_feature(&feature).await?;
                feature
            } else {
                bail!("expected existing feature `{item_id}` in collection `{collection_id}`")
            }
        }
    };

    // Add/update datetime
    let mut map = serde_json::Map::new();
    map.insert(
        "datetime".to_string(),
        json!(datetime.to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
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
        .unwrap_or_else(|| panic!("missing collection `{collection_id}`"));

    // Add/update asset
    collection.assets.insert(asset_id.to_string(), asset);

    // Write ollection
    db.update_collection(&collection).await
}

async fn load_items_from_object(
    key: &str,
    collection_id: &str,
    db: &Db,
    s3: &S3,
) -> anyhow::Result<()> {
    // Extract features
    let resp = s3.get_object(AWS_S3_BUCKET, key).await?;
    let data = resp.body.collect().await?.into_bytes();

    let mut features = if key.ends_with("json") {
        let value = serde_json::from_slice(&data)?;
        let fc = geojson::FeatureCollection::from_json_value(value)?;
        fc.features
    } else {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(&*data);

        let mut features = Vec::new();
        for result in rdr.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let record: crate::observation::Observation = result?;
            features.push(record.to_feature());
        }
        features
    };

    // Load features
    let now = std::time::Instant::now();
    let count = features.len();

    let proj = crate::proj::Proj::new("EPSG:2056", "EPSG:4326");

    sqlx::query(&format!(r#"TRUNCATE TABLE items."{}""#, collection_id))
        .execute(&db.pool)
        .await?;

    let mut ids_list = Vec::new();
    let mut properties_list = Vec::new();
    let mut assets_list = Vec::new();
    let mut geom_list = Vec::new();

    for (i, feature) in features.iter_mut().enumerate() {
        // id
        let id = if let Some(id) = &feature.id {
            match id {
                geojson::feature::Id::String(id) => id.to_owned(),
                geojson::feature::Id::Number(id) => id.to_string(),
            }
        } else {
            i.to_string()
        };
        ids_list.push(id.to_owned());

        // properties
        if let Some(properties) = feature.properties.as_mut() {
            properties.remove("description");
            if let Some(value) = properties.get("reference_ts") {
                if let Ok(s) = serde_json::from_value::<DateTime<Utc>>(value.to_owned()) {
                    properties.insert(
                        "datetime".to_string(),
                        json!(s.to_rfc3339_opts(SecondsFormat::Secs, true)),
                    );
                }
            }
        }
        properties_list.push(feature.properties.to_owned().map(sqlx::types::Json));

        // assets
        let asset = Asset::new(format!("{ROOT}/collections/{collection_id}/items/{id}"))
            .media_type(GEO_JSON)
            .roles(&["data"]);

        assets_list.push(sqlx::types::Json(json!({ id: asset })));

        // geom
        let mut geom: geo::Geometry = feature.geometry.to_owned().unwrap().try_into()?;
        geom.transform(&proj.0)?;
        geom_list.push(wkb::geom_to_wkb(&geom).unwrap());
    }

    bulk_load_items(
        collection_id,
        &ids_list,
        &properties_list,
        &geom_list,
        &assets_list,
        &db.pool,
    )
    .await?;

    // stats
    let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
    tracing::info!(
        "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
        count as f64 / elapsed
    );

    Ok(())
}

async fn bulk_load_items(
    collection: &str,
    ids: &[String],
    properties: &[Option<sqlx::types::Json<serde_json::Map<String, serde_json::Value>>>],
    geoms: &[Vec<u8>],
    assets: &[sqlx::types::Json<serde_json::Value>],
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    let batch_size = 10000;
    let total = geoms.len();

    let mut start = 0;
    let mut end = start + batch_size;

    let mut ids_batch;
    let mut properties_batch;
    let mut geoms_batch;
    let mut assets_batch;

    while start < total {
        if end < total {
            ids_batch = &ids[start..end];
            properties_batch = &properties[start..end];
            geoms_batch = &geoms[start..end];
            assets_batch = &assets[start..end];
        } else {
            ids_batch = &ids[start..];
            properties_batch = &properties[start..];
            geoms_batch = &geoms[start..];
            assets_batch = &assets[start..];
        }
        sqlx::query(&format!(
            r#"
            INSERT INTO items."{}" (id, properties, geom, assets)
            SELECT * FROM UNNEST($1::text[], $2::jsonb[], $3::bytea[], $4::jsonb[])
            "#,
            collection
        ))
        .bind(ids_batch)
        .bind(properties_batch)
        .bind(geoms_batch)
        .bind(assets_batch)
        .execute(pool)
        .await?;

        start = end;
        end += batch_size;
    }

    Ok(())
}

async fn copy_object(source: &str, target: &str, s3: &S3) -> anyhow::Result<()> {
    s3.client
        .copy_object()
        .copy_source(format!("{AWS_S3_BUCKET}/{source}"))
        .bucket(AWS_S3_BUCKET)
        .key(target)
        .acl(ObjectCannedAcl::PublicRead)
        .send()
        .await?;
    Ok(())
}
