import json
from pathlib import Path
import requests
import xml.etree.ElementTree as ET
from pyproj import Transformer

NS = {
    "che": "http://www.geocat.ch/2008/che",
    "gmd": "http://www.isotc211.org/2005/gmd",
    "gco": "http://www.isotc211.org/2005/gco",
}

# Extract collection info from metadata
def collection_from_geocat(uuid):
    r = requests.get(
        f"https://www.geocat.ch/geonetwork/srv/eng/xml.metadata.get?uuid={uuid}"
    )

    root = ET.fromstring(r.text)

    item = {
        "id": (
            e.text
            if (
                e := root.find(
                    ".//gmd:citation//gmd:identifier//gco:CharacterString", NS
                )
            )
            else uuid
        )
    }

    if e := root.find(".//gmd:citation//gmd:title//*[@locale='#EN']", NS):
        item["title"] = e.text

    if e := root.find(".//gmd:abstract//*[@locale='#EN']", NS):
        item["description"] = e.text

    item.update(
        {
            "extent": {
                "spatial": {"bbox": [[5.96, 45.82, 10.49, 47.81]]},
                "temporal": {"interval": [[None, None]]},
            },
            "license": "various",
            "links": [
                {
                    "href": f"https://www.geocat.ch/geonetwork/srv/eng/md.viewer#/full_view/{uuid}/tab/complete",
                    "rel": "metadata",
                    "type": "html/text",
                    "hreflang": "en",
                    "title": "Metadata (geocat.ch)",
                }
            ],
            "crs": ["http://www.opengis.net/def/crs/OGC/1.3/CRS84"],
        }
    )

    return item


uuids = [
    # "c994a2fa-047d-4a0d-841a-efe70c5815c8",
    # "b46a8f8d-bc48-41d3-b20a-de61d0763318",
    # "ad2b1452-9f3c-4137-9822-9758298bc025",
    # "1549b018-f8f0-4a56-bd17-c8a4377afe58", # klima
    # "e2e5132c-85df-417a-8706-f75068d4937e",
    # "a6296aa9-d183-45c3-90fc-f03ec7d637be",
    # "0a3b0af5-bbb4-4dde-bcff-adb27b932d77",
]

for uuid in uuids:
    try:
        collection = collection_from_geocat(uuid)
        path = Path(f"../collections/{collection['id']}.json")
        if path.exists():
            continue
        else:
            path.write_text(json.dumps(collection, indent=4))

    except Exception as e:
        print(e)

# Extract resources from file
path = Path("../data/ch.meteoschweiz.messnetz-klima_en.json")
file = json.loads(path.read_bytes())

collections = ["ch.meteoschweiz.tageswerte", "ch.meteoschweiz.stundenwerte"]

transformer = Transformer.from_crs("epsg:2056", "epsg:4326")

for feature in file["features"]:
    coordinates = feature["geometry"]["coordinates"]
    transformed = transformer.transform(coordinates[0], coordinates[1])[::-1]
    feature["geometry"]["coordinates"] = transformed

    properties = feature["properties"]
    properties.pop("description")

    for collection in collections:
        id = f"{collection.split('.')[2]}_{feature['id']}"

        item = {
            "id": id,
            "collection": collection,
            "geometry": feature["geometry"],
            "properties": {**properties, "datetime": "2022-06-08T09:32:38Z"},
            "bbox": [c for cs in [transformed, transformed] for c in cs],
        }

        p = Path(f"../collections/{collection}/items/{id}.json")
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps(item, indent=4))