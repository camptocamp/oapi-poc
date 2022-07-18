# STAC API and OGC API - Features - Part 1: Core

This API is an implementation of both the [STAC API](https://github.com/radiantearth/stac-api-spec) and the [OGC API - Features - Part 1: Core](https://ogcapi.ogc.org/features/) (OAFeat). The STAC API and OAFeat have a slightly different data model, which is briefly discussed below.

The STAC API is a “dataset based” download service providing access to packaged geospatial data and related metadata. The data model is based on the following main concepts:

- Collection: a `collection` is a set of metadata about a geospatial dataset, like its name, description, spatial and temporal extent, etc. Individual records within a collection are called `items`
- Item: an `item` represents an atomic collection of inseparable data and metadata. A STAC `item` is a GeoJSON Feature and can be easily read by any modern GIS or geospatial library, and it describes a SpatioTemporal Asset. This means that the GeoJSON is not the "actual" thing, but instead references files and serves as an index to an `asset`
- Asset: an `asset` is any file containing the actual data.

OAFeat is a "features based" service providing access to single objects (`features`) of a dataset. In OAFeat there is no concept of `asset` and the `item` is the actual data.

To summarize, the STAC API is aimed at providing access to the complete dataset, while OAFeat is aimed at providing access to single features of a dataset.

## API endpoints

The API provides the following endpoints:

| Endpoint                                        | Returns                                                 | Description                                                                         |
| ----------------------------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `/`                                             | JSON                                                    | Landing page                                                                        |
| `/conformance`                                  | JSON                                                    | Info about standards to which the API conforms                                      |
| `/collections`                                  | JSON                                                    | Object containing an array of Collection objects in the Catalog, and Link relations |
| `/collections/{collectionId}`                   | Collection                                              | Returns single Collection JSON                                                      |
| `/collections/{collectionId}/items`             | ItemCollection                                          | GeoJSON FeatureCollection-conformant entity of Item objects in collection           |
| `/collections/{collectionId}/items/{featureId}` | Item                                                    | Returns single Item (GeoJSON Feature)                                               |
| `/search`                                       | Item Collection                                         | STAC search endpoint                                                                |

The API provides currently the following datasets (collections):

- [Aktuelle Daten Temperatur](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/0a62455f-c39c-4084-bd54-36ee2192d3af) - ID: 0a62455f-c39c-4084-bd54-36ee2192d3af (OAFeat)
- [Climate normals temperature 1961-1990](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/ed6a30c9-672e-4d8f-95e4-8c5bef8ab417) - ID: ed6a30c9-672e-4d8f-95e4-8c5bef8ab417 (STAC)
- [CombiPrecip Niederschlagmenge akkumuliert 1h](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/e74c17ea-0822-44db-bef9-f37135a68245) - ID: e74c17ea-0822-44db-bef9-f37135a68245 (STAC)
- [CombiPrecip Niederschlagmenge akkumuliert 24h](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/7880287e-5d4b-4e15-b13f-846df89979a3) - ID: 7880287e-5d4b-4e15-b13f-846df89979a3 (STAC)
- [Gridded dataset of global radiation](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/4ccc5153-cc27-47b8-abee-9d6e12e19701) ID: 4ccc5153-cc27-47b8-abee-9d6e12e19701 (STAC)
- [Modelldaten COSMO-1E](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/a6296aa9-d183-45c3-90fc-f03ec7d637be) - ID: a6296aa9-d183-45c3-90fc-f03ec7d637be (STAC)
- [Radarprodukte](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/e2e5132c-85df-417a-8706-f75068d4937e) - ID: e2e5132c-85df-417a-8706-f75068d4937e (STAC)
- [Stundenwerte Messstationen](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/ad2b1452-9f3c-4137-9822-9758298bc025) - ID: ad2b1452-9f3c-4137-9822-9758298bc025 (OAFeat)
- [Tageswerte Messstationen](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/b46a8f8d-bc48-41d3-b20a-de61d0763318) - ID: b46a8f8d-bc48-41d3-b20a-de61d0763318 (STAC)
- [Warnungen vor Naturgefahren](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/35ff8133-364a-47eb-a145-0d641b706bff) - ID: 35ff8133-364a-47eb-a145-0d641b706bff (STAC)
- [Windprofilerdaten](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/collections/0a3b0af5-bbb4-4dde-bcff-adb27b932d77) - ID: 0a3b0af5-bbb4-4dde-bcff-adb27b932d77 (STAC)

