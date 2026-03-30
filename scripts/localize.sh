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
  "original_filenames": false,
  "bundle_structure": "%LANG_ISO%.lproj/Localizable.%FORMAT%",
  "exclude_tags": ["info_plist"]
}'
ios_plist_data='{
  "format": "strings",
  "export_empty_as": "base",
  "export_sort": "first_added",
  "original_filenames": false,
  "bundle_structure": "%LANG_ISO%.lproj/InfoPlist.%FORMAT%",
  "include_tags": ["info_plist"]
}'
ios_widget_data='{
  "format": "strings",
  "export_empty_as": "base",
  "export_sort": "first_added",
  "original_filenames": false,
  "bundle_structure": "%LANG_ISO%.lproj/Localizable.%FORMAT%",
  "include_tags": ["widget"]
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
    python3 -c '
import json
import sys

key = sys.argv[1]
raw = sys.stdin.read()

try:
    obj = json.loads(raw)
except json.JSONDecodeError as exc:
    print(f"Lokalise returned invalid JSON: {exc}: {raw}", file=sys.stderr)
    raise SystemExit(1)

if key not in obj:
    print(f"Lokalise response missing {key!r}: {json.dumps(obj)}", file=sys.stderr)
    raise SystemExit(1)

print(obj[key])
' "$key"
}

json_path() {
    path="${1}"
    python3 -c '
import json
import sys

path = sys.argv[1].split(".")
raw = sys.stdin.read()

try:
    obj = json.loads(raw)
except json.JSONDecodeError as exc:
    print(f"Lokalise returned invalid JSON: {exc}: {raw}", file=sys.stderr)
    raise SystemExit(1)

current = obj
for part in path:
    if not isinstance(current, dict) or part not in current:
        print("Lokalise response missing path {!r}: {}".format(".".join(path), json.dumps(obj)), file=sys.stderr)
        raise SystemExit(1)
    current = current[part]

print(current)
' "$path"
}

json_add_filter_langs() {
    langs="${1}"
    python3 -c 'import json, sys; obj = json.load(sys.stdin); obj["filter_langs"] = sys.argv[1].split(","); print(json.dumps(obj))' "$langs"
}

download_bundle() {
    temp_file=$(mktemp)
    data="${2}"
    langs="${5:-}"

    if [[ -n "$langs" ]]; then
        data=$(printf '%s' "$data" | json_add_filter_langs "$langs")
    fi

    echo "Downloading bundle ${1} config... to ${3}"

    process_id=$(curl --silent --show-error --request POST \
    --url "https://api.lokalise.com/api2/projects/$4/files/async-download" \
    --header "content-type: application/json" \
    --header "x-api-token: $token" \
    --data "$data" | json_obj_key "process_id"
    )

    process_url="https://api.lokalise.com/api2/projects/$4/processes/$process_id"
    bundle_url=""
    for _ in $(seq 1 120); do
        process_response=$(curl --silent --show-error \
            --url "$process_url" \
            --header "x-api-token: $token"
        )
        process_status=$(printf '%s' "$process_response" | json_path "process.status")

        if [[ "$process_status" == "finished" ]]; then
            bundle_url=$(printf '%s' "$process_response" | json_path "process.details.download_url")
            break
        fi

        if [[ "$process_status" == "failed" || "$process_status" == "cancelled" ]]; then
            echo "Lokalise async export failed: $process_response" >&2
            exit 1
        fi

        sleep 1
    done

    if [[ -z "$bundle_url" ]]; then
        echo "Timed out waiting for Lokalise async export: $process_id" >&2
        exit 1
    fi

    echo "Downloading ${1} bundle file..."
    curl --silent --show-error --location "$bundle_url" -o "$temp_file"

    echo "Unzipping ${1} bundle..."

    unzip -o -qq "$temp_file" -d "$3"

    echo "Localization update for ${1} complete"
}

check_localization() {
    matches=$(grep -rE '\*\* %[^*]+ \*\*' Packages/Localization --include="*.strings" 2>/dev/null || true)
    if [ -n "$matches" ]; then
        echo "⚠️  Found '** %@ **' pattern in localization files:"
        echo "$matches"
        exit 1
    fi
}

case $1 in
  "ios")
    download_bundle "ios" "$ios_data" "$2" "$mobile_project_id" "${3:-}"
    download_bundle "ios_plist" "$ios_plist_data" "$2" "$mobile_project_id" "${3:-}"
    download_bundle "ios_widget" "$ios_widget_data" "Packages/Localization/WidgetSources/Resources" "$mobile_project_id" "${3:-}"
    check_localization
  ;;
  "android")
    download_bundle "android" "$android_data" "$2" "$mobile_project_id" "${3:-}"
  ;;
  "core")
    download_bundle "core" "$core_data" "$2" "$core_project_id" "${3:-}"
  ;;
esac
