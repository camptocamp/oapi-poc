# OAPI - POC

Proof of concept (POC) to ingest geospatial datasets from [MeteoSwiss](https://www.meteoswiss.admin.ch/home.html) into a [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) and expose as [OGC API Features](https://ogcapi.ogc.org/features).

## Terms of Service

1. This service is **experimental and thus not for operational use**.
2. This service is limited in time, from 1.8.2022 to 31.10.2022 (might be extended).
3. The service has limited availability and limited operating hours: It is frequently rebooted.
4. During the limited service period, data can be accessed **for testing purposes only**. You must provide the source (author, title and link to the dataset).
5. Further, when using this service, the disclaimer of the Federal Administration and the respective terms of use must be complied with in every case. You should therefore read the disclaimer carefully: [disclaimer.admin.ch](http://disclaimer.admin.ch).

## Documentation

OGC API and STAC are designed to be explorable through `links` and a good starting point is the [`Landing Page (aka Root Catalog)`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/) which links to capabilities, descriptions and fundamental resources of the services.

The `OpenAPI` definition can be consumed through the [`SwaggerUI`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/swagger). Select the appropriate server and authorization (for endpoints except GET) to try it out.

Be aware that the api definition is not in sync with the service implementation. There are addinonally transactional endpints for `Collection` and `Feature/Item` resources and the schemas/definitions might diverge from the actual implementation.

## Usage

For now the basic use case is uploading a `STAC Asset` through the [`load-asset`](https://poc.meteoschweiz-poc.swisstopo.cloud/root/processes/load-asset) process. The input schema describes the json `body` of the `post` request passed to it's `./execute` endpoint. It requires the file as base64 encoded string, some asset properties, the collection id and the item id or an item object to create.

Example python scripts for loading an asset to an existing collection as well as extracting & creating a collection resource from a `geocat.ch` entry are in the [scripts](scripts) folder.

### Catalog Trees

Catalog trees can be created by adding collection resources with the property `type` set to `Catalog` and links with the relations `parent`, `child` and/or `item`. Naturally these relations should be reflected on the linked ressources as well.

## Consumption

The created resources can for example be consoumed with the [STAC Browser](https://radiantearth.github.io/stac-browser/#/external/poc.meteoschweiz-poc.swisstopo.cloud/root/). The assets contents accessible through the `href` reside on a [S3 bucket](http://met-oapi-poc.s3.amazonaws.com/).

### Tutorial

A  [TUTORIAL](https://github.com/camptocamp/oapi-poc/blob/main/tutorial/howto.md) is provided to integrate

- Complete dataset browsing  and donwload
- Feature data download via API with examples
- Integration in web and fat client applications

## Feedback / Survey

If you are interested in MeteoSwiss data or OGC API Features services, please answer our questions about the Proof of Concept.

Fill in our [SURVEY (DE)](https://de.surveymonkey.com/r/RL8HCBK?lang=de) or [SURVEY (EN)](https://de.surveymonkey.com/r/RL8HCBK?lang=en) which only takes about 10 min. Thank You!

## Questions?

Please drop us an e-mail to [customerservice@meteoswiss.ch](mailto:customerservice@meteoswiss.ch) with the subject `POC OGD24`.
