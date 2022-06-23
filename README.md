# OAPI - POC

Proof of concept (POC) to ingest geospatial datasets from [MeteoSuisse](https://www.meteoswiss.admin.ch/home.html) into a [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/), expose as [OGC API Features](https://ogcapi.ogc.org/features) and offer [OGC API Environmental Data Retrieval (EDR)](https://ogcapi.ogc.org/edr) capabilities.

## Documentation

OGC API and STAC are designed to be explorable through `links` and a good starting point is the [`Landing Page (aka Root Catalog)`](https://poc.meteoschweiz-poc.swisstopo.cloud/) which links to capabilities, descriptions and fundamental resources of the services.

The `OpenAPI` definition can be consumed through the [`SwaggerUI`](https://poc.meteoschweiz-poc.swisstopo.cloud/swagger). Select the appropriate server and authorization (for endpoints except GET) to try it out.

Be aware that the api definition is not in sync with the service implementation. There are addinonally transactional endpints for `Collection` and `Feature/Item` resources and the schemas/definitions might diverge from the actual implementation by the services.

## Usage

For now the basic use case is uploading a `STAC Asset` through the [`load-asset`](https://poc.meteoschweiz-poc.swisstopo.cloud/processes/load-asset) process. The input schema describes the json `body` of the `post` request passed to it's `./execute` endpoint. It requires the file as base64 encoded string, some asset properties, the collection id and the item id or an item object to create.

Example python scripts for loading an asset to an existing collection as well as extracting & creating a collection resource from a `geocat.ch` entry are in the [scripts](scripts) folder.

## Consumption

The created resources can for example be consoumed with the [STAC Browser](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/). The assets contents accessible through the `href` reside on a [S3 bucket](http://met-oapi-poc.s3.amazonaws.com/).

## Caveats

* Only the data on S3 is persisted, the rest lives in the containers of the docker composition.
* The asset content is not parsed, nor are any attributes derived.
* Only `base64` encoded string are supported as file value for now. Alternatively a `URI` to an acessible resource can be used to link or upload from a local file.
* An asset can be replaced by specifying the asset `id`, the S3 `key` and the `item id` as value in the inputs.
* It is currently not possible to updat the Feature/Item on upload.
* The `landing page` respectively `root catalog` is defined statically and loaded on startup/compilation.

## Catalog Trees

Catalog trees can be created by adding collection resources with the property `type` set to `Catalog` and links with the relations `parent`, `child` and/or `item`. Naturally these relations should be reflected on the linked ressources as well.
