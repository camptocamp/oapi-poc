import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
AUTH = ("user", "password")

# Dataset root (Landing Page / Root Catalog)
ROOT = "https://poc.meteoschweiz-poc.swisstopo.cloud"
# ROOT = "http://0.0.0.0:8484"

# Read existing collections
collections = requests.get(f"{ROOT}/collections").json()
ids = [c["id"] for c in collections["collections"]]

# Create or update collections and items
for path in Path("../collections").glob("*.json"):
    # Create or update collection
    collection = json.loads(path.read_bytes())
    id = collection["id"]

    assert path.stem == id, "Collection `id` must match file name"

    if id in ids:
        requests.put(
            f"{ROOT}/collections/{id}", auth=AUTH, json=collection
        ).raise_for_status()
    else:
        requests.post(
            f"{ROOT}/collections", auth=AUTH, json=collection
        ).raise_for_status()

    # Create or update items
    path = Path(f"../collections/{id}/items")

    if not path.exists():
        print(f"No items found for collection `{id}`, continue.")
        continue

    items = requests.get(f"{ROOT}/collections/{id}/items").json()
    fids = [f["id"] for f in items["features"]]

    for p in path.glob("*.json"):
        item = json.loads(p.read_bytes())

        assert item["collection"] == id, "Item `collection` must match folder name"

        if item["id"] in fids:
            requests.put(
                f"{ROOT}/collections/{id}/items/{item['id']}",
                auth=AUTH,
                json=item,
            )
        else:
            requests.post(f"{ROOT}/collections/{id}/items", auth=AUTH, json=item)

print("Collections & items updated successfully!")
