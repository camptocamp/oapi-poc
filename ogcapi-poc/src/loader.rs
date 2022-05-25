use anyhow::Context;
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

use ogcapi_drivers::s3::{ByteStream, S3};
use ogcapi_services::{Error, Processor, Result, State};
use ogcapi_types::{
    common::Crs,
    features::Feature,
    processes::{Execute, Process},
    stac::Asset,
};

static AWS_S3_BUCKET: &str = "met-oapi-poc";
static AWS_S3_BUCKET_BASE: &str = "http://met-oapi-poc.s3.amazonaws.com";

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
    item: Item,
}

#[derive(Deserialize, Debug, JsonSchema)]
struct File {
    /// Binary file data (base46 encoded)
    value: String,
    // encoding: Option<String>,
    /// Media Type of the file
    #[serde(rename = "mediaType")]
    media_type: String,
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

        // Setup S3 driver
        let s3 = S3::setup().await;

        // Upload asset
        let bytes = base64::decode(inputs.file.value).context("Failed to decode base64 string")?;
        s3.client
            .put_object()
            .bucket(AWS_S3_BUCKET)
            .key(&inputs.key)
            .body(ByteStream::from(bytes))
            .content_type(&inputs.file.media_type)
            .send()
            .await
            .context("Failed to put object to S3")?;

        let asset = Asset {
            href: format!(
                "{}/{}",
                AWS_S3_BUCKET_BASE,
                inputs.key.trim_start_matches('/')
            ),
            title: inputs.title,
            description: inputs.description,
            r#type: Some(inputs.file.media_type),
            roles: inputs.roles,
            additional_properties: Default::default(),
        };
        let key = inputs.id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let id = match inputs.item.value {
            ItemValue::String(id) => {
                let mut item = state
                    .drivers
                    .features
                    .read_feature(&inputs.collection, &id, &Crs::default())
                    .await?;

                item.assets.insert(key, asset);

                state.drivers.features.update_feature(&item).await?;

                id.to_owned()
            }
            ItemValue::Item(object) => {
                let mut item: Feature = serde_json::from_value(object.into())
                    .map_err(|e| Error::Exception(StatusCode::BAD_REQUEST, e.to_string()))?;

                item.assets.insert(key, asset);

                state.drivers.features.create_feature(&item).await?
            }
        };

        let location = url
            .join(&format!(
                "../../collections/{}/items/{}",
                &inputs.collection, id
            ))
            .unwrap();

        Ok(Json(location).into_response())
    }
}
