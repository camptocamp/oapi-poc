use std::path::Path;

use anyhow::bail;
use aws_sdk_s3::model::ObjectCannedAcl;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, SecondsFormat, Utc};
use geo::Transform;
use serde_json::json;

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
    let resp = s3.list_objects(AWS_S3_BUCKET, Some(prefix)).await?;

    // Register assets
    for object in resp.contents().unwrap_or_default() {
        // Last modified
        let datetime = object.last_modified.unwrap().to_chrono_utc();
        let age = now - datetime;

        // Source key
        let key = object.key().unwrap_or_default();
        if key.is_empty()
            || key.ends_with('/')
            || key.contains("/.")
            || (prefix.is_empty() && key.starts_with("mhs-upload"))
            || (!prefix.is_empty() && age.num_seconds() < 10)
            || (!prefix.is_empty() && age.num_seconds() >= 70 && age.num_minutes() % 5 != 0)
        {
            continue;
        }

        // Target key
        let target = key.trim_start_matches(prefix).trim_start_matches('/');

        // Collection id (uuid)
        let collection_id = target.split('/').next().unwrap_or_default();

        // Asset id (defaults to file name)
        let asset_id = Path::new(target)
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
            _ => None,
        };

        // Copy object
        if key != target {
            s3.client
                .copy_object()
                .copy_source(format!("{AWS_S3_BUCKET}/{key}"))
                .bucket(AWS_S3_BUCKET)
                .key(target)
                .acl(ObjectCannedAcl::PublicRead)
                .send()
                .await?;
        }

        // Update collection/item
        let result = match collection_id {
            "0a62455f-c39c-4084-bd54-36ee2192d3af" | "ad2b1452-9f3c-4137-9822-9758298bc025" => {
                if key.ends_with("ch.meteoschweiz.messwerte-lufttemperatur-10min_en.json")
                    || key.ends_with("observations-hourly.csv")
                {
                    load_items_from_object(key, collection_id, &db, &s3).await?;
                }
                asset_to_collection(collection_id, asset_id, asset, &db).await
            }
            _ => asset_to_item(collection_id, asset_id, asset, &datetime, &db).await,
        };

        // Cleanup
        match result {
            Ok(_) => {
                if !prefix.is_empty() {
                    s3.delete_object(AWS_S3_BUCKET, key).await?;
                }
            }
            Err(e) => {
                tracing::error!("failed to load asset: {}", e);
            }
        }
    }

    Ok(())
}

async fn asset_to_item(
    collection_id: &str,
    asset_id: &str,
    asset: Asset,
    datetime: &DateTime<Utc>,
    db: &Db,
) -> anyhow::Result<()> {
    // Get item (skip key if no mapping)
    let item_id = match collection_id {
        "e2e5132c-85df-417a-8706-f75068d4937e"
        | "e74c17ea-0822-44db-bef9-f37135a68245"
        | "7880287e-5d4b-4e15-b13f-846df89979a3" => "meteoswiss.radar.precip",
        "ed6a30c9-672e-4d8f-95e4-8c5bef8ab417" => "klimanormwerte.temperatur.1961-1990",
        "b46a8f8d-bc48-41d3-b20a-de61d0763318" => asset_id.split('_').nth(1).unwrap(),
        "4ccc5153-cc27-47b8-abee-9d6e12e19701" => &asset_id.split('_').last().unwrap()[..8],
        "35ff8133-364a-47eb-a145-0d641b706bff" => asset_id.trim_end_matches(".cap"),
        _ => bail!("no mapping for collection `{collection_id}`"),
    };

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
    let d = if collection_id == "35ff8133-364a-47eb-a145-0d641b706bff" {
        let parsed = DateTime::parse_from_str(
            &format!("{}+0000", asset_id.split('.').nth(2).unwrap()),
            "%Y%m%d%H%M%z",
        )?;
        parsed.to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        datetime.to_rfc3339_opts(SecondsFormat::Secs, true)
    };
    let mut map = serde_json::Map::new();
    map.insert("datetime".to_string(), json!(d));
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

    let mut tx = db.pool.begin().await?;

    sqlx::query(&format!(r#"TRUNCATE TABLE items."{}""#, collection_id))
        .execute(&mut tx)
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

    sqlx::query(&format!(
        r#"
        INSERT INTO items."{}" (id, properties, geom, assets)
        SELECT * FROM UNNEST ($1::text[], $2::jsonb[], $3::bytea[], $4::jsonb[])
        "#,
        collection_id
    ))
    .bind(&ids_list[..])
    .bind(&properties_list[..])
    .bind(&geom_list[..])
    .bind(&assets_list[..])
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    // stats
    let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
    tracing::info!(
        "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
        count as f64 / elapsed
    );

    Ok(())
}
