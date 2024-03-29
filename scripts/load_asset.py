import base64
import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
auth = ("user", "password")

# API endpoint
url = "https://poc.meteoschweiz-poc.swisstopo.cloud/root"
# url = "http://0.0.0.0:8484/root"

# Process description
# r = requests.get(f'{url}/processes/load-asset')
# print(json.dumps(r.json(), indent=4, sort_keys=True))

# Process execution
path = Path("../data/observations-hourly.csv")

with path.open("rb") as file:
    inputs = {
        "inputs": {
            "file": {
                "value": base64.b64encode(file.read()).decode("utf-8"),
                # "value": {
                #     "uri": "https://www.meteoswiss.admin.ch/content/dam/meteoswiss/en/service-und-publikationen/produkt/doc/standardservices.pdf",
                #     "method": "link"
                # },
                "mediaType": "text/csv",
            },
            "key": f"mhs-upload/ad2b1452-9f3c-4137-9822-9758298bc025/{path.name}",
            "id": f"{path.name}",
            "collection": "ad2b1452-9f3c-4137-9822-9758298bc025",
            "item": {
                "value": {
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
                    }
                }
                # "value": "ARO"
            },
        }
    }

    r = requests.post(f"{url}/processes/load-asset/execution", auth=auth, json=inputs)
    print(r.text)
