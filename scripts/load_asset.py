import base64
import json
from pathlib import Path
import requests

# Creadentials (to be replaced)
auth = ('user', 'password')

# Asset loader process endpoint
url = 'https://poc.meteoschweiz-poc.swisstopo.cloud/processes/load-asset'

# Process description
r = requests.get(url)
print(json.dumps(r.json(), indent=4, sort_keys=True))

# Process execution
path = Path('../data/observations-hourly_excerpt.csv')

with path.open('rb') as file:
    r = requests.post(f'{url}/execution', auth=auth,
                      json={'inputs': {
                                'file': {
                                    'value': base64.b64encode(file.read()).decode('utf-8'),
                                    'mediaType': 'text/csv',
                                    },
                                'key': path.name,
                                'collection': 'countries',
                                'item': '4'}
                            })
    r.text
