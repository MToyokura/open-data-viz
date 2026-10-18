# 配信データの公開手順

この文書は `data_preparation/output/` で生成したデータを Cloud Storage に配置し、GitHub Pages のフロントエンドから配信する手順を説明します。公開するファイルはフロントエンドが読むものに限ります。`output/` の各プロジェクト直下には API 応答、取得時メタデータ、監査用ファイルも含まれるため、プロジェクトディレクトリ全体を配信先へ同期しないでください。

## 公開するファイル

| データ | `output/` 以下の公開対象 | 配信先の prefix |
| --- | --- | --- |
| 純資産分布 | `asset_distribution/manifest.json`、`asset_distribution/items/` | `asset_distribution/` |
| 小売物価 | `commodity_prices/meta_data/cat02_lookup.json`、`area_lookup.json`、`commodity_prices/transformed_data/` | `commodity_prices/` |
| 年齢階級・雇用形態・教育・性別別の雇用者数 | `workers_by_age_employment_type_education_sex/meta_data/options.json`、`transformed_data/` | `workers_by_age_employment_type_education_sex/` |
| 野菜の産地リレー | `vegetable_production_relay/products.json`、`*.summary.json`、`veg-*/details/` | `vegetable_production_relay/` |
| 都道府県地図 | `prefectures/N03-20260101_prefecture_simplified.geojson` | `prefectures/` |

この一覧は現在のフロントエンドの読み込み先に合わせています。読み込み先や生成形式を変更した場合は、公開対象も合わせて更新してください。

## 公開先と認証

リポジトリのルートにある `.env` に、GCS バケット URI を `GCS_BUCKET` として設定します。`.env` は Git 管理対象外です。バケット URI や個人のアカウント名、アクセストークンなどをこの文書や公開リポジトリに書かないでください。

GitHub Pages のビルドは GitHub リポジトリ変数 `VITE_STORAGE_BASE_URL` を使います。これはブラウザーからアクセスする配信元の HTTP(S) ベース URL です。バケット URI と対応する公開 URL をリポジトリ設定に登録してください。データの GCS 配置と GitHub Pages の公開は別々の処理です。

公開操作を行う前に、GCS への書き込み権限を持つ `gcloud` 認証を済ませてください。認証情報を `.env` に保存したり、コマンド出力や文書に貼り付けたりしないでください。`gcloud` CLI と GNU gzip のバージョンを記録し、同じ prefix への公開を同時に実行しないでください。

```sh
gcloud version
gzip --version
```

## 公開用ディレクトリの作成

以下はリポジトリルートから Bash または Zsh で実行します。全件の例は表に記載した全データが生成済みであることを前提にしています。特定のデータだけを公開する場合は、そのデータのコピー処理と同期・検証呼び出しだけを残してください。未生成データのコピーを省略したまま呼び出すと必要な公開ファイルが抜けるため、必須ファイルと件数を確認してください。

```sh
set -euo pipefail

source .env
: "${GCS_BUCKET:?ルートの .env に GCS_BUCKET を設定してください}"
case "$GCS_BUCKET" in
  gs://*) ;;
  *) echo "GCS_BUCKET は gs:// で始まる URI にしてください" >&2; exit 1 ;;
esac

OUTPUT=./data_preparation/output
WORK_DIR="$(mktemp -d)"
PUBLISH_DIR="$WORK_DIR/publish"
STAGE_DIR="$WORK_DIR/gzip"
VERIFY_DIR="$WORK_DIR/verify"
BACKUP_DIR="$WORK_DIR/remote-backup"
cleanup() {
  status=$?
  trap - EXIT
  if [ "$status" -eq 0 ]; then
    rm -rf "$WORK_DIR"
  else
    printf '公開処理が失敗しました。復旧用バックアップを保持しています: %s\n' "$WORK_DIR" >&2
  fi
  exit "$status"
}
trap cleanup EXIT

mkdir -p \
  "$PUBLISH_DIR/asset_distribution/items" \
  "$PUBLISH_DIR/commodity_prices/meta_data" \
  "$PUBLISH_DIR/workers_by_age_employment_type_education_sex/meta_data" \
  "$PUBLISH_DIR/vegetable_production_relay" \
  "$PUBLISH_DIR/prefectures" \
  "$VERIFY_DIR" \
  "$BACKUP_DIR"

# 純資産分布: manifest と値チャンクだけ
cp "$OUTPUT/asset_distribution/manifest.json" \
  "$PUBLISH_DIR/asset_distribution/manifest.json"
cp -a "$OUTPUT/asset_distribution/items/." \
  "$PUBLISH_DIR/asset_distribution/items/"

# 小売物価: 画面が使う分類辞書と加工済み系列だけ
cp "$OUTPUT/commodity_prices/meta_data/cat02_lookup.json" \
  "$PUBLISH_DIR/commodity_prices/meta_data/"
cp "$OUTPUT/commodity_prices/meta_data/area_lookup.json" \
  "$PUBLISH_DIR/commodity_prices/meta_data/"
cp -a "$OUTPUT/commodity_prices/transformed_data" \
  "$PUBLISH_DIR/commodity_prices/"

# 雇用者数: 画面が使う選択肢と加工済み系列だけ
cp "$OUTPUT/workers_by_age_employment_type_education_sex/meta_data/options.json" \
  "$PUBLISH_DIR/workers_by_age_employment_type_education_sex/meta_data/"
cp -a "$OUTPUT/workers_by_age_employment_type_education_sex/transformed_data" \
  "$PUBLISH_DIR/workers_by_age_employment_type_education_sex/"

# 野菜の産地リレー: 索引、集計、都道府県詳細だけ
cp "$OUTPUT/vegetable_production_relay/products.json" \
  "$PUBLISH_DIR/vegetable_production_relay/"
cp "$OUTPUT/vegetable_production_relay"/*.summary.json \
  "$PUBLISH_DIR/vegetable_production_relay/"
for detail_dir in "$OUTPUT/vegetable_production_relay"/veg-*/details; do
  product_key="${detail_dir%/details}"
  product_key="${product_key##*/}"
  mkdir -p "$PUBLISH_DIR/vegetable_production_relay/$product_key"
  cp -a "$detail_dir" "$PUBLISH_DIR/vegetable_production_relay/$product_key/"
done

# 都道府県地図: 簡略化済み GeoJSON だけ
cp "$OUTPUT/prefectures/N03-20260101_prefecture_simplified.geojson" \
  "$PUBLISH_DIR/prefectures/"

# 公開対象のファイル一覧を確認
find "$PUBLISH_DIR" -type f | sort

# 必須ファイルと件数を確認する。件数が前回から減った場合は理由を確認するまで停止する。
test -s "$PUBLISH_DIR/asset_distribution/manifest.json"
test -n "$(find "$PUBLISH_DIR/asset_distribution/items" -type f -name '*.json' -print -quit)"
test -s "$PUBLISH_DIR/commodity_prices/meta_data/cat02_lookup.json"
test -s "$PUBLISH_DIR/commodity_prices/meta_data/area_lookup.json"
test -s "$PUBLISH_DIR/workers_by_age_employment_type_education_sex/meta_data/options.json"
test -s "$PUBLISH_DIR/vegetable_production_relay/products.json"
test -s "$PUBLISH_DIR/prefectures/N03-20260101_prefecture_simplified.geojson"
find "$PUBLISH_DIR" -type f | sort > "$WORK_DIR/publish-files.txt"
wc -l "$WORK_DIR/publish-files.txt"
# 削除許可一覧は公開対象一覧と別に管理する。gzip 移行時は空のままにする。
: > "$WORK_DIR/delete-allowlist.txt"

# 上書き前に対象オブジェクトの本体とメタデータを退避する。
while IFS= read -r local_file; do
  relative="${local_file#"$PUBLISH_DIR/"}"
  remote="$GCS_BUCKET/$relative"
  mkdir -p "$BACKUP_DIR/$(dirname "$relative")"
  gcloud storage objects describe "$remote" --format=json \
    > "$BACKUP_DIR/$relative.metadata.json"
  gcloud storage cp --do-not-decompress "$remote" "$BACKUP_DIR/$relative"
done < "$WORK_DIR/publish-files.txt"

# gzip -n -6 を使い、元のファイル名・拡張子を保った圧縮ステージを作る。
cargo run --quiet --manifest-path data_preparation/Cargo.toml -- \
  gzip-stage "$PUBLISH_DIR" "$STAGE_DIR"
find "$STAGE_DIR" -type f | sort
```

バックアップは各公開対象の既存 GCS 本体と JSON メタデータで、gzip 自動展開を無効にして保存しています。この段階では読み取りだけです。ファイル一覧に API の `raw_data/`、取得メタデータ、監査 JSON などが含まれていないことを確認します。公開と画面確認の完了後にシェルを正常終了すると一時ディレクトリを削除します。エラーで終了した場合は復旧用バックアップを残すため、確認が終わるまで手動で削除しないでください。

## gzip 移行前の記録

上のバックアップに保存した各 `*.metadata.json` から既存 `Cache-Control` と内容のサイズ・チェックサムを記録します。メタデータが未設定の場合は代表 URL の HTTP 応答も確認して実効のキャッシュ方針を特定します。prefix ごとの従来方針を確認し、`no-transform` を加えた値を同期時の `CACHE_CONTROL` として確定します。方針が複数なら値ごとに同期を分けます。移行前の本体・メタデータ記録と `PUBLISH_DIR` を確認完了まで保持してください。`WORK_DIR` はエラー時には自動削除されません。

この手順は公開対象をローカルにコピーした後、JSON と GeoJSON のみから gzip ステージを作ります。空ファイル、JSON として不正なファイル、許可外の拡張子、空のステージは停止条件です。GNU gzip の実装とバージョンはステージ作成時に記録します。

## 同期と検証

アップロード前に、対象 prefix を公開対象一覧に固定し、公開元ファイルと gzip ステージの件数・相対パスが一致することを確認します。キャッシュ方針と gzip ステージのチェックサム記録も保存します。純資産分布のチャンクは `items/` prefix のみに同期します。同期コマンドに削除フラグを付けず、マニフェストはチャンクの検証が完了するまで更新しません。

次の環境変数を移行前の記録から設定してください。各 prefix 内で従来のキャッシュ方針が複数の場合は、この一括同期を分け、各グループに対応する値を使います。

```sh
: "${COMMODITY_CACHE_CONTROL:?既存方針と no-transform を設定してください}"
: "${WORKERS_CACHE_CONTROL:?既存方針と no-transform を設定してください}"
: "${VEGETABLE_CACHE_CONTROL:?既存方針と no-transform を設定してください}"
: "${PREFECTURES_CACHE_CONTROL:?既存方針と no-transform を設定してください}"
: "${ASSET_CACHE_CONTROL:?既存方針と no-transform を設定してください}"
```

例えば既存方針が `public, max-age=3600` なら `public, max-age=3600, no-transform` です。全 prefix に機械的に同一の値を仮定しないでください。

```sh
sync_and_verify() {
  data_id="$1"
  content_type="$2"
  local_dir="$PUBLISH_DIR/$data_id"
  stage_dir="$STAGE_DIR/$data_id"
  remote_prefix="$GCS_BUCKET/$data_id"
  remote_dir="$remote_prefix/"
  verify_dir="$VERIFY_DIR/$data_id"
  cache_control="$3"

  rm -rf "$verify_dir"
  gcloud storage rsync --recursive --checksums-only \
    --content-type="$content_type" --content-encoding=gzip \
    --cache-control="$cache_control" \
    "$stage_dir/" "$remote_dir"
  cargo run --quiet --manifest-path data_preparation/Cargo.toml -- gzip-reconcile \
    --stage-dir "$stage_dir" --source-dir "$local_dir" \
    --remote-prefix "$remote_prefix" --verification-dir "$verify_dir" \
    --content-type "$content_type" --cache-control "$cache_control"
}

sync_and_verify commodity_prices application/json "$COMMODITY_CACHE_CONTROL"
sync_and_verify workers_by_age_employment_type_education_sex application/json "$WORKERS_CACHE_CONTROL"
sync_and_verify vegetable_production_relay application/json "$VEGETABLE_CACHE_CONTROL"
sync_and_verify prefectures application/geo+json "$PREFECTURES_CACHE_CONTROL"
```

`gzip-reconcile` はステージ中の公開対象ファイルだけを処理します。各オブジェクトについて `gcloud storage objects describe` から `size`・`crc32c`・`generation`・`metageneration`・Content-Type・Content-Encoding・Cache-Control を取得し、ローカルの圧縮ファイルとサイズ・CRC32C を照合します。判定不能なら停止します。内容一致かつメタデータ不一致の場合のみ、取得した generation と metageneration を条件に `gcloud storage objects update` で修復します。内容不一致なら、generation 条件付きでステージから `gcloud storage cp` し直します。同期で転送が省略されたオブジェクトも必ず調べます。修復後は `gcloud storage cp --do-not-decompress` で保存バイト列を取得し、ステージとの一致、gzip 展開結果、メタデータを再検証します。ステージにないオブジェクトを列挙・修復・削除せず、競合や検証失敗はエラーで停止します。

純資産分布は schema version 3 の manifest とチャンクを公開します。フロントエンドも version 3 のみを読み込み、旧 manifest との互換性はありません。通常の更新では、参照先の切り替えを一貫させるため、チャンクを先に配置・検証し、manifest は最後に置き換えます。manifest の置き換え中に旧形式と新形式を切り替える移行作業はありません。

```sh
ASSET_LOCAL="$PUBLISH_DIR/asset_distribution"
ASSET_STAGE="$STAGE_DIR/asset_distribution"
ASSET_REMOTE="$GCS_BUCKET/asset_distribution"
ASSET_VERIFY="$VERIFY_DIR/asset_distribution"

gcloud storage rsync --recursive --checksums-only \
  --content-type=application/json --content-encoding=gzip \
  --cache-control="$ASSET_CACHE_CONTROL" \
  "$ASSET_STAGE/items/" "$ASSET_REMOTE/items/"
rm -rf "$ASSET_VERIFY/items"
cargo run --quiet --manifest-path data_preparation/Cargo.toml -- gzip-reconcile \
  --stage-dir "$ASSET_STAGE/items" --source-dir "$ASSET_LOCAL/items" \
  --remote-prefix "$ASSET_REMOTE/items" \
  --verification-dir "$ASSET_VERIFY/items" \
  --content-type application/json --cache-control "$ASSET_CACHE_CONTROL"

# manifest はチャンクの修復・検証が終わった後にだけ更新する。
mkdir -p "$WORK_DIR/asset-manifest/stage" "$WORK_DIR/asset-manifest/source"
cp "$ASSET_STAGE/manifest.json" "$WORK_DIR/asset-manifest/stage/manifest.json"
cp "$ASSET_LOCAL/manifest.json" "$WORK_DIR/asset-manifest/source/manifest.json"
cargo run --quiet --manifest-path data_preparation/Cargo.toml -- gzip-reconcile \
  --stage-dir "$WORK_DIR/asset-manifest/stage" \
  --source-dir "$WORK_DIR/asset-manifest/source" \
  --remote-prefix "$ASSET_REMOTE" \
  --verification-dir "$ASSET_VERIFY/manifest" \
  --content-type application/json --cache-control "$ASSET_CACHE_CONTROL"
```

検証では各公開対象を明示的な `gcloud storage cp --do-not-decompress` で空の検証用ディレクトリに取得します。ダウンロードした gzip のチェックサムと圧縮ステージが一致し、gzip 展開結果が元ファイルと一致し、オブジェクトのメタデータが期待値と一致することを各ファイルで確認します。検証用ディレクトリは新規でなければなりません。gcloud の自動展開を防ぐため、検証用ダウンロードの cp には `--do-not-decompress` を必ず付けます。

各 prefix の同期・検証が失敗した場合は公開作業を停止します。内容、メタデータ、復元可能なバックアップが確認できるまで次の prefix や manifest の更新に進まないでください。`no-transform` は GCS 側の decompressive transcoding を防ぐための指定です。gcloud の `--do-not-decompress` はダウンロードクライアント側だけの指定です。

削除はアップロードと分離します。同期後、`gcloud storage rsync --dry-run --delete-unmatched-destination-objects` を個々の prefix に対して実行し、削除候補をレビューしてください。raw data、監査ファイル、manifest 参照中のチャンクが候補に含まれた場合は停止します。gzip 移行では `$WORK_DIR/delete-allowlist.txt` を空のままにし、削除しません。将来削除が必要な場合は、公開対象一覧とは別の許可一覧に `gs://.../object<TAB>generation` と理由を記録し、dry-run 候補と完全一致するものだけを対象にします。

```sh
# 許可一覧で明示したオブジェクトのみ実行する。generation が変われば停止する。
gcloud storage rm "$REMOTE_OBJECT" --if-generation-match="$GENERATION"
```

純資産分布の古いチャンクは、旧 manifest のキャッシュ期間と開いたままの画面の利用期間を考慮した保持期間が定まるまで維持します。

### 復旧

画面や HTTP 応答の確認が終わるまで、`remote-backup/` に保存した本体と移行前メタデータを保持します。復旧するオブジェクトは許可一覧に明記し、保存した元の本体を再アップロードします。元の Content-Type、Content-Encoding の有無、Cache-Control を JSON 記録から復元します。元が未圧縮だった場合は `Content-Encoding` を指定せず、圧縮本体から `Content-Encoding` だけを取り除く状態を作らないでください。純資産分布ではチャンクを戻して検証してから manifest を戻します。復旧後に保存内容、メタデータ、HTTP 応答、画面を再確認し、キャッシュ済み応答も確認してください。

```sh
restore_metadata=()
if [ -n "$ORIGINAL_CONTENT_TYPE" ]; then
  restore_metadata+=("--content-type=$ORIGINAL_CONTENT_TYPE")
fi
if [ -n "$ORIGINAL_CACHE_CONTROL" ]; then
  restore_metadata+=("--cache-control=$ORIGINAL_CACHE_CONTROL")
fi
if [ -n "$ORIGINAL_CONTENT_ENCODING" ]; then
  restore_metadata+=("--content-encoding=$ORIGINAL_CONTENT_ENCODING")
fi
gcloud storage cp --do-not-decompress "$BACKUP_FILE" "$REMOTE_OBJECT" \
  "${restore_metadata[@]}" --if-generation-match="$CURRENT_GENERATION"
gcloud storage objects describe "$REMOTE_OBJECT" --format=json
```

復旧時も保存内容・HTTP 応答・画面の確認に失敗したら後続の公開へ進まず、原因を解消して再検証します。通常のアプリは JSON 全体を `fetch()` し、Range 要求を使いません。圧縮オブジェクトの Range は通常の部分取得にならない場合があるため、Range を使う読み込みへ変更する場合は別途配信方式を検討してください。

## フロントエンドの公開と画面確認

データを配置・検証した後、フロントエンドを `main` に反映します。純資産分布の公開データとフロントエンドはどちらも schema version 3 を使用します。`.github/workflows/deploy-sveltekit.yml` は `main` への push で GitHub Pages をビルド・公開します。workflow が参照する GitHub リポジトリ変数 `VITE_STORAGE_BASE_URL` が正しい配信元を指すことも確認してください。workflow は GCS へデータをアップロードしません。

Actions の deploy job が成功したら、キャッシュ無効の状態で公開ページを開き、Network パネルで manifest、必要なチャンク、小売物価、雇用者数、野菜リレー、都道府県 GeoJSON の応答を確認します。応答に `Content-Encoding: gzip` があり、Network の転送サイズがリソースサイズより小さいことを記録してください。保存サイズだけを転送量として扱わないでください。JSON が通常の `fetch()` / `response.json()` で読み込まれることも画面で確かめます。

各データについて初期読み込み、絞り込み、系列比較、詳細表示を確認します。純資産分布では初期 manifest と必要なチャンク、野菜リレーでは一覧・集計・都道府県詳細を含めて確認します。その後、通常キャッシュを使うブラウザーでも表示を確認します。古い未圧縮応答がキャッシュされている期間は、すべての要求がすぐに gzip 応答になるとはみなしません。

- 純資産分布: manifest と必要チャンクを読み、初期表示・属性変更・複数系列比較を確認する。
- 小売物価: 分類辞書と地域辞書を読み、データ絞り込みと系列比較を確認する。
- 雇用者数: options と系列データを読み、属性絞り込みと系列比較を確認する。
- 野菜リレー: 商品一覧、集計、都道府県詳細を確認する。
- 都道府県地図: GeoJSON を読み、地図表示を確認する。
- 各データが JSON として解析され、Network パネルの応答ヘッダーと転送サイズを記録している。
- 開いたままにしていたページは再読み込みして最新のデータを読み込む。

## データ prefix と公開範囲

アップロードと削除は別の操作です。バケットルートやチャンクの親 prefix に削除付き同期を実行しないでください。削除候補は dry-run と明示的な許可一覧でレビューし、raw data や監査ファイルが候補にあれば停止します。今回の gzip 移行では削除しません。
