import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
AUTH = ("user", "password")

# Dataset root (Landing Page / Root Catalog)
ROOT = "https://poc.meteoschweiz-poc.swisstopo.cloud"
# ROOT = "http://0.0.0.0:8484"

# Create or update collections and items
for path in Path("../collections").glob("*.json"):
    # Create or update collection
    collection = json.loads(path.read_bytes())
    collection_id = collection["id"]

    assert path.stem == collection_id, "Collection `id` must match file name"

    if requests.get(f"{ROOT}/collections/{collection_id}").ok:
        requests.put(
            f"{ROOT}/collections/{collection_id}", auth=AUTH, json=collection
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

        if requests.get(f"{ROOT}/collections/{collection_id}/items/{item_id}").ok:
            requests.put(
                f"{ROOT}/collections/{collection_id}/items/{item['id']}",
                auth=AUTH,
                json=item,
            )
        else:
            requests.post(
                f"{ROOT}/collections/{collection_id}/items", auth=AUTH, json=item
            )

print("Collections & items updated successfully!")
