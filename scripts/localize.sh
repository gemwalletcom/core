#!/bin/bash

set -e

# public read only
token="d27a64f96c062117da75b13ce34519a84a89d203"
project_id="94865410644ee707546334.60736699"
# iOS
ios_data='{
  "format": "strings",
  "export_empty_as": "base",
  "export_sort": "first_added",
  "bundle_structure": "%LANG_ISO%.lproj/Localizable.%FORMAT%"
}'
# Android
android_data='{
  "format": "xml",
  "export_empty_as": "base",
  "export_sort": "first_added",
  "bundle_structure": "values_%LANG_ISO%/strings.%FORMAT%"
}'

# android_localization_dir="android/app/src/main/res" #! unused for now.

json_obj_key() {
    key="${1}"
    python3 -c "import json,sys;obj=json.load(sys.stdin); print(obj[\"${key}\"]);"
}

download_bundle() {
    temp_file=$(mktemp)
    echo "Downloading bundle ${1} config..."
    
    bundle_url=$(curl --silent --request POST \
    --url https://api.lokalise.com/api2/projects/$project_id/files/download \
    --header "content-type: application/json" \
    --header "x-api-token: $token" \
    --data "${2}" | json_obj_key "bundle_url"
    )
    
    echo "Downloading ${1} bundle file..."
    curl --silent $bundle_url -o ${temp_file}

    echo "Unzipping ${1} bundle..."

    unzip -o -qq ${temp_file} -d ${3}

    echo "Localization update for ${1} complete"
}

case $1 in
  "ios")
    download_bundle "ios" "$ios_data" $2
  ;;
  "android")
    download_bundle "android" "$android_data" $2
  ;;
esac
