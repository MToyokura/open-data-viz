#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi
: "${GCS_BUCKET:?Set GCS_BUCKET to a gs:// bucket or prefix in .env or the environment}"
case "$GCS_BUCKET" in
  gs://*) ;;
  *) echo "GCS_BUCKET must start with gs://" >&2; exit 1 ;;
esac
GCS_BUCKET="${GCS_BUCKET%/}"
command -v gcloud >/dev/null || { echo "gcloud is required" >&2; exit 1; }
command -v gzip >/dev/null || { echo "gzip is required" >&2; exit 1; }

OUTPUT="$ROOT/data_preparation/output"
WORK_DIR="$(mktemp -d)"
PUBLISH_DIR="$WORK_DIR/publish"
trap 'rm -rf "$WORK_DIR"' EXIT

copy_file() {
  local source="$1" destination="$2"
  [[ -s "$source" ]] || { echo "Missing or empty required file: $source" >&2; return 1; }
  mkdir -p "$(dirname "$PUBLISH_DIR/$destination")"
  cp "$source" "$PUBLISH_DIR/$destination"
}

copy_json_tree() {
  local source="$1" destination="$2" found=0 file relative
  [[ -d "$source" ]] || { echo "Missing required directory: $source" >&2; return 1; }
  while IFS= read -r -d '' file; do
    found=1
    relative="${file#"$source"/}"
    mkdir -p "$PUBLISH_DIR/$destination/$(dirname "$relative")"
    cp "$file" "$PUBLISH_DIR/$destination/$relative"
  done < <(find "$source" -type f \( -name '*.json' -o -name '*.geojson' \) -print0)
  (( found )) || { echo "No JSON or GeoJSON files in $source" >&2; return 1; }
}

copy_file "$OUTPUT/asset_distribution/manifest.json" asset_distribution/manifest.json
copy_json_tree "$OUTPUT/asset_distribution/items" asset_distribution/items
copy_file "$OUTPUT/commodity_prices/meta_data/cat02_lookup.json" commodity_prices/meta_data/cat02_lookup.json
copy_file "$OUTPUT/commodity_prices/meta_data/area_lookup.json" commodity_prices/meta_data/area_lookup.json
copy_json_tree "$OUTPUT/commodity_prices/transformed_data" commodity_prices/transformed_data
copy_file "$OUTPUT/workers_by_age_employment_type_education_sex/meta_data/options.json" workers_by_age_employment_type_education_sex/meta_data/options.json
copy_json_tree "$OUTPUT/workers_by_age_employment_type_education_sex/transformed_data" workers_by_age_employment_type_education_sex/transformed_data
copy_file "$OUTPUT/vegetable_production_relay/products.json" vegetable_production_relay/products.json

shopt -s nullglob
summaries=("$OUTPUT/vegetable_production_relay"/*.summary.json)
details=("$OUTPUT/vegetable_production_relay"/veg-*/details)
shopt -u nullglob
(( ${#summaries[@]} > 0 )) || { echo "No vegetable summary files found" >&2; exit 1; }
(( ${#details[@]} > 0 )) || { echo "No vegetable detail directories found" >&2; exit 1; }
for file in "${summaries[@]}"; do
  copy_file "$file" "vegetable_production_relay/$(basename "$file")"
done
for directory in "${details[@]}"; do
  product="$(basename "$(dirname "$directory")")"
  copy_json_tree "$directory" "vegetable_production_relay/$product/details"
done
copy_file "$OUTPUT/prefectures/N03-20260101_prefecture_simplified.geojson" prefectures/N03-20260101_prefecture_simplified.geojson

mapfile -d '' files < <(find "$PUBLISH_DIR" -type f \( -name '*.json' -o -name '*.geojson' \) -print0)
(( ${#files[@]} > 0 )) || { echo "No publishable files were collected" >&2; exit 1; }
printf 'Publishing %d files:\n' "${#files[@]}"
for file in "${files[@]}"; do
  gzip -n -6 -c "$file" > "$file.gz"
  mv "$file.gz" "$file"
done

printf 'Syncing JSON files to %s...\n' "$GCS_BUCKET"
gcloud storage rsync "$PUBLISH_DIR" "$GCS_BUCKET" \
  --recursive \
  --checksums-only \
  --exclude='.*\.geojson$' \
  --content-type=application/json \
  --content-encoding=gzip \
  --cache-control='public, max-age=300, no-transform'

printf 'Syncing GeoJSON files to %s...\n' "$GCS_BUCKET"
gcloud storage rsync "$PUBLISH_DIR" "$GCS_BUCKET" \
  --recursive \
  --checksums-only \
  --exclude='.*\.json$' \
  --content-type=application/geo+json \
  --content-encoding=gzip \
  --cache-control='public, max-age=300, no-transform'

echo 'Publishing complete.'
