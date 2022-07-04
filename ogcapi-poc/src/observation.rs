use serde::Deserialize;
use serde_json::{json, Map};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Observation {
    datetime: String,
    value: f64,
    station_name: String,
    nat_abbr: String,
    wigos_id: String,
    e_coord: f64,
    n_coord: f64,
    latitude: f64,
    longitude: f64,
    #[serde(deserialize_with = "csv::invalid_option")]
    installation_height: Option<f64>,
    param_short: String,
}

impl Observation {
    pub(crate) fn to_feature(&self) -> geojson::Feature {
        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(geojson::Value::Point(vec![
                self.e_coord,
                self.n_coord,
            ]))),
            id: Some(geojson::feature::Id::String(format!(
                "{}_{}_{}",
                self.nat_abbr, self.param_short, self.datetime
            ))),
            properties: Some(Map::from_iter([
                ("datetime".to_string(), json!(self.datetime)),
                ("value".to_string(), json!(self.value)),
                ("station_name".to_string(), json!(self.station_name)),
                ("nat_abbr".to_string(), json!(self.nat_abbr)),
                ("wigos_id".to_string(), json!(self.wigos_id)),
                (
                    "installation_height".to_string(),
                    json!(self.installation_height),
                ),
                ("param_short".to_string(), json!(self.param_short)),
            ])),
            foreign_members: None,
        }
    }
}
