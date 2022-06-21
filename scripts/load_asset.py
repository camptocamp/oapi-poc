import base64
import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
auth = ("user", "password")

# API endpoint
url = "https://poc.meteoschweiz-poc.swisstopo.cloud"
# url = "http://0.0.0.0:8484"

# Process description
# r = requests.get(f'{url}/processes/load-asset')
# print(json.dumps(r.json(), indent=4, sort_keys=True))

# Process execution
path = Path("../data/ch.meteoschweiz.messwerte-lufttemperatur-10min_en.json")

with path.open("rb") as file:
    inputs = {
        "inputs": {
            "file": {
                "value": base64.b64encode(file.read()).decode("utf-8"),
                # "value": {
                #     "uri": "https://www.meteoswiss.admin.ch/content/dam/meteoswiss/en/service-und-publikationen/produkt/doc/standardservices.pdf",
                #     "method": "link"
                # },
                "mediaType": "application/pdf",
            },
            "key": f"mhs-upload/0a62455f-c39c-4084-bd54-36ee2192d3af/{path.name}",
            "id": f"{path.stem}",
            "collection": "0a62455f-c39c-4084-bd54-36ee2192d3af",
            "item": {
                # "value": {
                #     "geometry": {
                #         "type": "Polygon",
                #         "coordinates": [
                #             [
                #                 [5.96, 45.82],
                #                 [10.49, 45.82],
                #                 [10.49, 47.81],
                #                 [5.96, 47.81],
                #                 [5.96, 45.82],
                #             ]
                #         ],
                #     }
                # }
                "value": "messwerte-lufttemperatur-10min"
            },
        }
    }

    r = requests.post(f"{url}/processes/load-asset/execution", auth=auth, json=inputs)
    print(r.text)
