#!/bin/bash

# Usage: `bash ./load_observations_current.sh path/to/file.txt datetime`
#
# TODO:
# - credential handling

# Arguments
file=${1}
datetime=${2:-"$(date --utc +%Y-%m-%dT%H:%M:%SZ)"}

mediatype="application/json"
collection="ch.meteoschweiz.messwerte-lufttemperatur-10min"
item="messwerte-lufttemperatur-10min"

# Asset id = file name without extension
id=$(basename "$file" ".${file##*.}")

# Asset key for S3
key="$collection/$item/$(basename "$file")"

# Execute body
body="{
    \"inputs\": {
        \"file\": {
            \"value\": \"$(base64 --wrap=0 "$file")\",
            \"mediaType\": \"$mediatype\"
        },
        \"key\": \"$key\",
        \"id\": \"$id\",
        \"roles\": [\"data\"],
        \"collection\": \"$collection\",
        \"item\": {
            \"value\": \"$item\"
        },
        \"properties\": {
            \"value\": {
                \"datetime\": \"$datetime\"
            }
        }
    }
}"

# Create/update item/asset
ROOT="https://poc.meteoschweiz-poc.swisstopo.cloud/root"
# ROOT="http://0.0.0.0:8484/root"

curl -X POST "$ROOT/processes/load-asset/execution" \
    -u "user:password" \
    -H "Content-Type: application/json" \
    -d @- "$HOST" <<CURL_DATA
$body
CURL_DATA
