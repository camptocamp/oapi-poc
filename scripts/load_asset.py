import base64
import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
auth = ('user', 'password')

# API endpoint
url = 'https://poc.meteoschweiz-poc.swisstopo.cloud'
# url = 'http://0.0.0.0:8484'

# Process description
# r = requests.get(url)
# print(json.dumps(r.json(), indent=4, sort_keys=True))

# Process execution
path = Path('../data/observations-hourly_excerpt.csv')

with path.open('rb') as file:
    inputs = {
        'inputs': {
            'file': {
                'value': base64.b64encode(file.read()).decode('utf-8'),
                'mediaType': 'text/csv',
            },
            'key': f"test/{path.name}",
            'collection': 'ch.meteoschweiz.klimanormwerte-temperatur_1961_1990',
            'item': {
                'value': {
                    'geometry': {
                        "type": "Polygon",
                        "coordinates": [[[5.96, 45.82], [10.49, 45.82], [10.49, 47.81], [5.96, 47.81], [5.96, 45.82]]],
                    }
                }
            }
        }
    }

    r = requests.post(
        f'{url}/processes/load-asset/execution', auth=auth, json=inputs)
    print(r.text)
