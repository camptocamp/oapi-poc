import requests
import xml.etree.ElementTree as ET

NS = {
    'che': 'http://www.geocat.ch/2008/che',
    'gmd': 'http://www.isotc211.org/2005/gmd',
    'gco': 'http://www.isotc211.org/2005/gco',
}

# Creadentials (to be replaced)
AUTH = ('user', 'password')

# Extract collection info from metadata
def collection_from_geocat(uuid):
    r = requests.get(f'https://www.geocat.ch/geonetwork/srv/eng/xml.metadata.get?uuid={uuid}')

    root = ET.fromstring(r.text)

    return {
        'id': root.find(".//gmd:citation//gmd:identifier//gco:CharacterString", NS).text,
        'title': root.find(".//gmd:citation//gmd:title//*[@locale='#EN']", NS).text,
        'description': root.find(".//gmd:abstract//*[@locale='#EN']", NS).text,
        'extent': {
            'spatial': {
                'bbox': [[5.96, 45.82, 10.49, 47.81]]
            },
            'temporal': {
                'interval': [[None, None]]
            }
        },
        'license': 'various',
        'links': [{
            'href': f'https://www.geocat.ch/geonetwork/srv/eng/md.viewer#/full_view/{uuid}/tab/complete',
            'rel': 'metadata',
            'type': 'html/text',
            'hreflang': 'en',
            'title': 'Metadata (geocat.ch)'
        }],
        'crs': ['http://www.opengis.net/def/crs/OGC/1.3/CRS84']
    }


# Load collection
uuid = '1549b018-f8f0-4a56-bd17-c8a4377afe58'

collection = collection_from_geocat(uuid)

r = requests.delete(f"https://poc.meteoschweiz-poc.swisstopo.cloud/collections/{collection['id']}", auth=AUTH)
r.status_code
r = requests.post("https://poc.meteoschweiz-poc.swisstopo.cloud/collections", auth=AUTH, json=collection)
r.status_code
# r.text