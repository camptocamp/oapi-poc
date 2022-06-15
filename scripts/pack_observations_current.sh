#!/bin/bash

# Usage: `bash ./pack_observations_current.sh path/to/file.txt datetime`

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

# Pack file
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

echo "$body" > "${file%.*}_execute.json"
