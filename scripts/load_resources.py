import collections
import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
AUTH = ("user", "password")

# Dataset root (Landing Page / Root Catalog)
ROOT = "https://poc.meteoschweiz-poc.swisstopo.cloud/root"
# ROOT = "http://0.0.0.0:8484/root"

def update(d, u):
    """Update two dictionaries recursively"""
    for k, v in u.items():
        if isinstance(v, collections.abc.Mapping):
            d[k] = update(d.get(k, {}), v)
        else:
            d[k] = v
    return d


# Create or update collections and items
for path in Path("../collections").glob("*.json"):
    # Create or update collection
    collection = json.loads(path.read_bytes())
    collection_id = collection["id"]

    assert path.stem == collection_id, "Collection `id` must match file name"

    response = requests.get(f"{ROOT}/collections/{collection_id}")
    if response.ok:
        requests.put(
            f"{ROOT}/collections/{collection_id}",
            auth=AUTH,
            json=update(response.json(), collection),
        ).raise_for_status()
    else:
        requests.post(
            f"{ROOT}/collections", auth=AUTH, json=collection
        ).raise_for_status()

    # Create or update items
    for p in path.parent.glob(f"{collection_id}/items/*.json"):
        item = json.loads(p.read_bytes())
        item_id = item["id"]

        assert (
            item["collection"] == collection_id
        ), "Item `collection` must match folder name"

        response = requests.get(f"{ROOT}/collections/{collection_id}/items/{item_id}")
        if response.ok:
            requests.put(
                f"{ROOT}/collections/{collection_id}/items/{item['id']}",
                auth=AUTH,
                json=update(response.json(), item),
            )
        else:
            requests.post(
                f"{ROOT}/collections/{collection_id}/items", auth=AUTH, json=item
            )

print("Collections & items updated successfully!")
