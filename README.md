# OAPI - POC

Proof of concept (POC) to ingest geospatial datasets from [MetoSuisse](https://www.meteoswiss.admin.ch/home.html) into a [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/), expose as [OGC API Features](https://ogcapi.ogc.org/features) and offer [OGC API Environmental Data Retrieval (EDR)](https://ogcapi.ogc.org/edr) capabilities.

## Documentation

OGC API and STAC are designed to be explorable through `links` and a good starting point is the [`Landing Page (aka Root Catalog)`](https://poc.meteoschweiz-poc.swisstopo.cloud/) which links to capabilities, descriptions and fundamental resources of the services.

The `OpenAPI` definition can be consumed through the [`SwaggerUI`](https://poc.meteoschweiz-poc.swisstopo.cloud/swagger). Select the appropriate server and authorization (for endpoints except GET) to try it out.

Be aware that the api definition is not in sync with the service implementation. There are addinonally transactional endpints for `Collection` and `Feature/Item` resources and the schemas/definitions might diverge from the actual implementation by the services.

## Usage

For now the basic use case is uploading a `STAC Asset` through the [`load-asset`](https://poc.meteoschweiz-poc.swisstopo.cloud/processes/load-asset) process. The input schema describes the json `body` of the `post` request passed to it's `./execute` endpoint. It requires the file as base64 encoded string, some asset properties, the collection id and the item id or an item object to create.

Example python scripts for loading an asset to an existing collection as well as extracting & creating a collection resource from a `geocat.ch` entry are in the [scripts](scripts) folder.

## Consumption

The created resources can for example be consoumed with the [STAC Browser](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/). Currently the assets are loaded into a minion container, in the future they will reside on a [S3 bucket](http://met-oapi-poc.s3.amazonaws.com/).

## Caveats

* No data is persisted, it lives in the containers of the docker composition.
* The asset content is not parsed, nor are any attributes derived.
* Only `base64` encoded string are supported as file value for now.
* An asset can be replaced by specifying the asset `id`, the S3 `key` and the `item id` as value in the inputs.
* It is currently not possible to updat the Feature/Item on upload.
* Catalog trees can only be created from the collection downwards as the root catalog is immutable for now.

## Catalgo Trees (advanced & untested)

Potentially catalog trees can be setup by inserting collection resources with the property `type` set to `Catalog` that feature the proper link relations `parent` and `child` and/or `item`. Naturally this relation should be reflected on the linke ressources as well.
