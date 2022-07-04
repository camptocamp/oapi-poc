import json
from pathlib import Path
import requests
import xml.etree.ElementTree as ET

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

    item = {"id": uuid}
    # item = {
    #     "id": (
    #         e.text
    #         if (
    #             e := root.find(
    #                 ".//gmd:citation//gmd:identifier//gco:CharacterString", NS
    #             )
    #             is not None
    #         )
    #         else uuid
    #     )
    # }

    if (e := root.find(".//gmd:citation//gmd:title//*[@locale='#EN']", NS)) is not None:
        item["title"] = e.text

    if (e := root.find(".//gmd:abstract//*[@locale='#EN']", NS)) is not None:
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
    "0a62455f-c39c-4084-bd54-36ee2192d3af",
    "b46a8f8d-bc48-41d3-b20a-de61d0763318",
    "ad2b1452-9f3c-4137-9822-9758298bc025",
    "4ccc5153-cc27-47b8-abee-9d6e12e19701",
    "e2e5132c-85df-417a-8706-f75068d4937e",
    "e74c17ea-0822-44db-bef9-f37135a68245",
    "7880287e-5d4b-4e15-b13f-846df89979a3",
    "a6296aa9-d183-45c3-90fc-f03ec7d637be",
    "0a3b0af5-bbb4-4dde-bcff-adb27b932d77",
    "35ff8133-364a-47eb-a145-0d641b706bff",
    "ed6a30c9-672e-4d8f-95e4-8c5bef8ab417",
]

for uuid in uuids:
    try:
        collection = collection_from_geocat(uuid)
        path = Path(f"../collections/{collection['id']}.json")
        if path.exists() and False:
            continue
        else:
            path.write_bytes(
                json.dumps(collection, indent=4, ensure_ascii=False).encode("utf8")
            )

    except Exception as e:
        print(e)

# Extract resources from file
collection = "b46a8f8d-bc48-41d3-b20a-de61d0763318"

path = Path("../data/observations-hourly.csv")

for line in path.open("r").readlines()[1:]:
    parts = line.split(";")

    id = parts[3]

    lat, lng = float(parts[7]), float(parts[8])

    item = {
        "id": id,
        "collection": collection,
        "geometry": {"type": "Point", "coordinates": [lng, lat]},
        "properties": {
            "station_name": parts[2],
            "nat_abbr": parts[3],
            "wigos_id": parts[4],
        },
        "bbox": [lng, lat, lng, lat],
    }

    p = Path(f"../collections/{collection}/items/{id}.json")
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(item, indent=4, ensure_ascii=False))
