# OAPI - POC

Proof of concept (POC) to ingest geospatial datasets from [MeteoSuisse](https://www.meteoswiss.admin.ch/home.html) into a [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/), expose as [OGC API Features](https://ogcapi.ogc.org/features) and offer [OGC API Environmental Data Retrieval (EDR)](https://ogcapi.ogc.org/edr) capabilities.

## Documentation

OGC API and STAC are designed to be explorable through `links` and a good starting point is the [`Landing Page (aka Root Catalog)`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/) which links to capabilities, descriptions and fundamental resources of the services.

The `OpenAPI` definition can be consumed through the [`SwaggerUI`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/swagger). Select the appropriate server and authorization (for endpoints except GET) to try it out.

Be aware that the api definition is not in sync with the service implementation. There are addinonally transactional endpints for `Collection` and `Feature/Item` resources and the schemas/definitions might diverge from the actual implementation by the services.

## Usage

For now the basic use case is uploading a `STAC Asset` through the [`load-asset`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/processes/load-asset) process. The input schema describes the json `body` of the `post` request passed to it's `./execute` endpoint. It requires the file as base64 encoded string, some asset properties, the collection id and the item id or an item object to create.

Example python scripts for loading an asset to an existing collection as well as extracting & creating a collection resource from a `geocat.ch` entry are in the [scripts](scripts) folder.

## Consumption

The created resources can for example be consoumed with the [STAC Browser](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/). The assets contents accessible through the `href` reside on a [S3 bucket](http://met-oapi-poc.s3.amazonaws.com/).

## Catalog Trees

Catalog trees can be created by adding collection resources with the property `type` set to `Catalog` and links with the relations `parent`, `child` and/or `item`. Naturally these relations should be reflected on the linked ressources as well.
