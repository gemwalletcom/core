#!/bin/bash

set -e

if [[ -z "$LOCALIZE_TOKEN" ]]; then
    echo "Error: LOCALIZE_TOKEN is not set!"
    exit 1
fi

token=$LOCALIZE_TOKEN
mobile_project_id="94865410644ee707546334.60736699"
core_project_id="2608747066be591cd57427.16218028"

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
# Core
core_data='{
"format": "properties",
"export_empty_as": "base",
"export_sort": "first_added",
"original_filenames": false,
"language_mapping": [
    {
      "original_language_iso": "zh_CN",
      "custom_language_iso": "zh-Hans"
    },
    {
      "original_language_iso": "zh_TW",
      "custom_language_iso": "zh-Hant"
    },
    {
      "original_language_iso": "pt_BR",
      "custom_language_iso": "pt-BR"
    }
],
"bundle_structure": "%LANG_ISO%/localizer.ftl"
}'

json_obj_key() {
    key="${1}"
    python3 -c "import json,sys;obj=json.load(sys.stdin); print(obj[\"${key}\"]);"
}

download_bundle() {
    temp_file=$(mktemp)
    echo "Downloading bundle ${1} config..."

    bundle_url=$(curl --silent --request POST \
    --url https://api.lokalise.com/api2/projects/$4/files/download \
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
    download_bundle "ios" "$ios_data" $2 $mobile_project_id
  ;;
  "android")
    download_bundle "android" "$android_data" $2 $mobile_project_id
  ;;
  "core")
    download_bundle "core" "$core_data" $2 $core_project_id
  ;;
esac
