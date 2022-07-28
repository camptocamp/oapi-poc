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

    collection = {"id": uuid}
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
        collection["title"] = e.text

    if (e := root.find(".//gmd:abstract//*[@locale='#EN']", NS)) is not None:
        collection["description"] = e.text

    collection.update(
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

    return collection


uuids = [
    "0a62455f-c39c-4084-bd54-36ee2192d3af",
    "b46a8f8d-bc48-41d3-b20a-de61d0763318",
    "ad2b1452-9f3c-4137-9822-9758298bc025",
    "4ccc5153-cc27-47b8-abee-9d6e12e19701",
    "e2e5132c-85df-417a-8706-f75068d4937e",
    "e74c17ea-0822-44db-bef9-f37135a68245",
    "7880287e-5d4b-4e15-b13f-846df89979a3",
    "a6296aa9-d183-45c3-90fc-f03ec7d637be",
    # "0a3b0af5-bbb4-4dde-bcff-adb27b932d77",
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
    }

    p = Path(f"../collections/{collection}/items/{id}.json")
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(item, indent=4, ensure_ascii=False))


# Model catalog tree
def add_child(catalogs, parent_id, child_id, child_title):
    catalogs[child_id] = {
        **catalogs[parent_id],
        "type": "Catalog",
        "id": child_id,
        "title": child_title,
        "links": [],
    }

    catalogs[parent_id]["links"].append(
        {
            "href": child_id,
            "rel": "child",
            "type": "application/json",
            "title": catalogs[child_id]["title"],
        }
    )

    catalogs[child_id]["links"].append(
        {
            "href": parent_id,
            "rel": "parent",
            "type": "application/json",
            "title": catalogs[parent_id]["title"],
        }
    )


# root catalog (collection)
collection_id = "a6296aa9-d183-45c3-90fc-f03ec7d637be"

collection = json.load(Path(f"../collections/{collection_id}.json").open())

catalogs = {collection_id: collection}

# create child catalogs
format_id = "cosmo-1e_grib2"
add_child(catalogs, collection_id, format_id, "COSMO-1E - GRIB2")

zz = ["00", "03", "06", "09", "12", "15", "18", "21"]
hhh = [str(item).zfill(3) for item in range(0, 34)]
parameter_leveltype = {
    "T_2M": "single-level",
    "TOT_PREC": "single-level",
    "T_level_2000m": "heightamsl-level",
    "T_level_78": "model-level",
    "T_level_79": "model-level",
    "T_level_80": "model-level",
    "T_level_500hPa": "pressure-level",
}
member = [str(item).zfill(3) for item in range(0, 11)]

for z in zz:
    z_id = f"{format_id}_{z}"
    add_child(catalogs, format_id, z_id, f"COSMO-1E - GRIB2 - {z}")
    for h in hhh:
        h_id = f"{z_id}_{h}"
        add_child(catalogs, z_id, h_id, f"COSMO-1E - GRIB2 - {z} - {h}")
        for param, l in parameter_leveltype.items():
            param_id = f"{h_id}_{param}"
            add_child(
                catalogs, h_id, param_id, f"COSMO-1E - GRIB2 - {z} - {h} - {param}"
            )

            # add items
            for m in member:
                id = f"COSMO-1E_alps_rotlatlon_{l}_leadtime_{h}_member_{m}_parameter_{param}"
                item = {
                    "id": id,
                    "collection": collection_id,
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [
                            [
                                [5.96, 45.82],
                                [10.49, 45.82],
                                [10.49, 47.81],
                                [5.96, 47.81],
                                [5.96, 45.82],
                            ]
                        ],
                    },
                    "properties": {
                        "model-name": "COSMO-1E",
                        "domain": "alps",
                        "grid-type": "rotlatlon",
                        "level-type": l,
                        "leadtime": h,
                        "member": m,
                        "parameter": param,
                        "parameter-shortname": param.split("_level")[0],
                    },
                    "links": [
                        {
                            "href": f"../../{param_id}",
                            "rel": "parent",
                            "type": "application/json",
                            "title": catalogs[param_id]["title"],
                        }
                    ],
                }

                if "level" in param:
                    item["properties"]["level-value"] = param.split("level_")[1]

                p = Path(f"../collections/{collection_id}/items/{id}.json")
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text(json.dumps(item, indent=4, ensure_ascii=False))

                catalogs[param_id]["links"].append(
                    {
                        "href": f"{collection_id}/items/{id}",
                        "rel": "item",
                        "type": "application/geo+json",
                    }
                )

for catalog_id, catalog in catalogs.items():
    p = Path(f"../collections/{catalog_id}.json")
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(catalog, indent=4, ensure_ascii=False))
