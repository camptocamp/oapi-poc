use anyhow::Context;
use aws_sdk_s3::model::ObjectCannedAcl;
use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::{gen::SchemaSettings, JsonSchema};
use serde::Deserialize;
use serde_json::{Map, Value};
use url::Url;
use uuid::Uuid;

use ogcapi_drivers::s3::ByteStream;
use ogcapi_services::{Error, Processor, Result, State};
use ogcapi_types::{
    common::Crs,
    features::Feature,
    processes::{Execute, Process},
    stac::Asset,
};

use crate::{AWS_S3_BUCKET, AWS_S3_BUCKET_BASE};

/// STAC Asset loader
pub(crate) struct AssetLoader;

/// Asset loader input schema
#[derive(Deserialize, Debug, JsonSchema)]
struct AssetLoaderInputs {
    /// File to upload
    file: File,
    /// S3 key
    key: String,
    /// Optional asset id
    id: Option<String>,
    /// The displayed title for clients and users.
    title: Option<String>,
    /// A description of the Asset providing additional details, such as how it was processed or created.
    description: Option<String>,
    /// The semantic roles of the asset.
    #[serde(default)]
    roles: Vec<String>,
    /// Collection `id`
    collection: String,
    /// Item object to create or existing Item `id`
    item: Option<Item>,
    /// Preperties to update
    properties: Option<Properties>,
}

#[derive(Deserialize, Debug, JsonSchema)]
struct File {
    /// File
    value: FileValue,
    // encoding: Option<String>,
    /// Media Type of the file
    #[serde(rename = "mediaType")]
    media_type: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(untagged)]
enum FileValue {
    /// File content (base46 encoded)
    Value(String),
    /// File uri
    Reference(FileReference),
}

#[derive(Deserialize, Debug, JsonSchema)]
struct FileReference {
    uri: String,
    method: Method,
}

/// Load or link the referenced filed
#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Method {
    Link,
    Load,
}

#[derive(Deserialize, Debug, JsonSchema)]
struct Item {
    value: ItemValue,
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(untagged)]
enum ItemValue {
    /// Existing Item id
    String(String),
    /// An Item/Feature to create
    Item(Map<String, Value>),
}
#[derive(Deserialize, Debug, JsonSchema)]
struct Properties {
    value: Map<String, Value>,
}

/// Asset loader output schema "URI of the crated/updated Item."
#[derive(JsonSchema)]
struct AssetLoaderOutputs(String);

#[async_trait]
impl Processor for AssetLoader {
    fn id(&self) -> String {
        "load-asset".to_string()
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
            &serde_json::to_value(
                &gen.clone()
                    .into_root_schema_for::<AssetLoaderInputs>()
                    .schema,
            )
            .unwrap(),
            &serde_json::to_value(&gen.into_root_schema_for::<AssetLoaderOutputs>().schema)
                .unwrap(),
        )
    }

    async fn execute(&self, execute: Execute, state: &State, url: &Url) -> Result<Response> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: AssetLoaderInputs = serde_json::from_value(value)
            .map_err(|e| Error::Exception(StatusCode::BAD_REQUEST, e.to_string()))?;

        // Create asset
        let mut asset = match inputs.file.value {
            FileValue::Value(v) => {
                let bytes = base64::decode(v).context("Failed to decode base64 string")?;
                state
                    .s3
                    .client
                    .put_object()
                    .bucket(AWS_S3_BUCKET)
                    .key(&inputs.key)
                    .body(ByteStream::from(bytes))
                    .content_type(&inputs.file.media_type)
                    .acl(ObjectCannedAcl::PublicRead)
                    .send()
                    .await
                    .context("Failed to put object to S3")?;

                Asset::new(format!(
                    "{}/{}",
                    AWS_S3_BUCKET_BASE,
                    inputs.key.trim_start_matches('/')
                ))
            }
            FileValue::Reference(reference) => match reference.method {
                Method::Link => Asset::new(reference.uri),
                Method::Load => {
                    let stream = if reference.uri.starts_with("http") {
                        let resp = reqwest::get(reference.uri).await.expect("request failed");
                        ByteStream::from(resp.bytes().await.unwrap())
                    } else {
                        ByteStream::from_path(reference.uri).await.unwrap()
                    };
                    state
                        .s3
                        .client
                        .put_object()
                        .bucket(AWS_S3_BUCKET)
                        .key(&inputs.key)
                        .body(stream)
                        .content_type(&inputs.file.media_type)
                        .acl(ObjectCannedAcl::PublicRead)
                        .send()
                        .await
                        .context("Failed to put object to S3")?;

                    Asset::new(format!(
                        "{}/{}",
                        AWS_S3_BUCKET_BASE,
                        inputs.key.trim_start_matches('/')
                    ))
                }
            },
        };

        asset.title = inputs.title;
        asset.description = inputs.description;
        asset.r#type = Some(inputs.file.media_type);
        asset.roles = inputs.roles;

        let key = inputs.id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let location = if let Some(item) = inputs.item {
            let item_id = match item.value {
                ItemValue::String(id) => {
                    let mut item = state
                        .drivers
                        .features
                        .read_feature(&inputs.collection, &id, &Crs::default())
                        .await?
                        .expect("existing item");

                    item.assets.insert(key, asset);

                    if let Some(properties) = inputs.properties {
                        item.append_properties(properties.value)
                    }

                    state.drivers.features.update_feature(&item).await?;

                    id.to_owned()
                }
                ItemValue::Item(object) => {
                    let mut item: Feature = serde_json::from_value(object.into())
                        .map_err(|e| Error::Exception(StatusCode::BAD_REQUEST, e.to_string()))?;

                    item.assets.insert(key, asset);
                    item.collection = Some(inputs.collection.to_owned());

                    if state
                        .drivers
                        .features
                        .read_feature(
                            &inputs.collection,
                            &item.id.clone().unwrap_or_default(),
                            &Crs::default(),
                        )
                        .await?
                        .is_some()
                    {
                        state.drivers.features.update_feature(&item).await?;
                        item.id.unwrap()
                    } else {
                        state.drivers.features.create_feature(&item).await?
                    }
                }
            };

            format!("../../collections/{}/items/{}", &inputs.collection, item_id)
        } else {
            format!("../../collections/{}", &inputs.collection)
        };

        let location = url.join(&location).unwrap();

        Ok(Json(location).into_response())
    }
}
