# OAPI - POC

Proof of concept (POC) to ingest geospatial datasets from [MeteoSwiss](https://www.meteoswiss.admin.ch/home.html) into a [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) and expose as [OGC API Features](https://ogcapi.ogc.org/features).

## OGC API Server

Server: `https://poc.meteoschweiz-poc.swisstopo.cloud/root/`

> **Warning**
> - This is service is *experimental and not for operational use*. 
> - *Limited service period*: from 1.8.2022 to 30.9.2022 might be extend
> - *Limited availability / operating hours*: Server is frequently rebooted
> - *Limited availability*: Server is frequently rebooted
> - *General terms of use*: 
>   - When using this service , the disclaimer of the Federal administration and the respective terms of use must be complied with in every case. You should therefore read the disclaimer carefully to ensure that you comply with the terms of use and the disclaimer.admin.ch.
>   - During the the limitied service period, data can be accessed for **testing purposes** only. You must provide the source (author, title and link to the dataset).

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

## Tutorial

A  [TUTORIAL](https://github.com/camptocamp/oapi-poc/blob/main/tutorial/howto.md) is provided to integrate
- Complete dataset browsing  and donwload
- Feature data download via API with examples 
- Integration in web and fat client applications

## Feedback required: Can you answer our questions about the Proof of Concept?
We are inviting you to take our survey about our Proof of Concept  data and services since you are interested in MeteoSwiss data or OGC API Features services.

-> Fill out the [SURVEY](https://de.surveymonkey.com/r/RL8HCBK) in DE, it takes only 10 min


