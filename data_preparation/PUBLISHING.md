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

公開操作を行う前に、GCS への書き込み権限を持つ `gcloud` 認証を済ませてください。認証情報を `.env` に保存したり、コマンド出力や文書に貼り付けたりしないでください。

## 公開用ディレクトリの作成

以下はリポジトリルートから Bash または Zsh で実行します。全件の例は表に記載した全データが生成済みであることを前提にしています。特定のデータだけを公開する場合は、そのデータのコピー処理と同期・検証呼び出しだけを残してください。未生成データのコピーを省略したまま同期呼び出しを残すと、空のステージ内容に合わせて配信済みファイルを削除するおそれがあります。

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
VERIFY_DIR="$WORK_DIR/verify"
trap 'rm -rf "$WORK_DIR"' EXIT

mkdir -p \
  "$PUBLISH_DIR/asset_distribution/items" \
  "$PUBLISH_DIR/commodity_prices/meta_data" \
  "$PUBLISH_DIR/workers_by_age_employment_type_education_sex/meta_data" \
  "$PUBLISH_DIR/vegetable_production_relay" \
  "$PUBLISH_DIR/prefectures" \
  "$VERIFY_DIR"

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
```

この段階では GCS を変更していません。ファイル一覧に API の `raw_data/`、取得メタデータ、監査 JSON などが含まれていないことを確認します。一時ディレクトリはシェルを終了すると削除されるため、公開と検証が終わるまで同じターミナルを使ってください。

## 同期と検証

最初に、純資産分布以外の更新対象を prefix ごとに同期し、GCS から取り直した内容を照合します。`--delete-unmatched-destination-objects` は対象 prefix 内のローカルに存在しないオブジェクトを削除し、以前の全体同期で混入した取得データなどもその prefix から取り除きます。公開用ディレクトリを確認してから実行してください。

```sh
sync_and_verify() {
  data_id="$1"
  local_dir="$PUBLISH_DIR/$data_id"
  remote_dir="$GCS_BUCKET/$data_id/"
  verify_dir="$VERIFY_DIR/$data_id"

  mkdir -p "$verify_dir"
  gcloud storage rsync --recursive --delete-unmatched-destination-objects \
    "$local_dir/" "$remote_dir"
  gcloud storage rsync --recursive "$remote_dir" "$verify_dir/"
  diff -qr "$local_dir" "$verify_dir"
}

sync_and_verify commodity_prices
sync_and_verify workers_by_age_employment_type_education_sex
sync_and_verify vegetable_production_relay
sync_and_verify prefectures
```

純資産分布は schema version 3 の manifest とチャンクを公開します。フロントエンドも version 3 のみを読み込み、旧 manifest との互換性はありません。通常の更新では、参照先の切り替えを一貫させるため、チャンクを先に配置・検証し、manifest は最後に置き換えます。manifest の置き換え中に旧形式と新形式を切り替える移行作業はありません。

```sh
ASSET_LOCAL="$PUBLISH_DIR/asset_distribution"
ASSET_REMOTE="$GCS_BUCKET/asset_distribution"
ASSET_VERIFY="$VERIFY_DIR/asset_distribution"
mkdir -p "$ASSET_VERIFY/items"

gcloud storage rsync --recursive --delete-unmatched-destination-objects \
  "$ASSET_LOCAL/items/" "$ASSET_REMOTE/items/"
gcloud storage rsync --recursive \
  "$ASSET_REMOTE/items/" "$ASSET_VERIFY/items/"
diff -qr "$ASSET_LOCAL/items" "$ASSET_VERIFY/items"

gcloud storage cp "$ASSET_LOCAL/manifest.json" \
  "$ASSET_REMOTE/manifest.json"
gcloud storage cp "$ASSET_REMOTE/manifest.json" \
  "$ASSET_VERIFY/manifest.json"
cmp "$ASSET_LOCAL/manifest.json" "$ASSET_VERIFY/manifest.json"
```

`diff` と `cmp` が何も表示せずに終了すれば、ローカルと配信先のファイルが一致しています。終了コードが 0 でない場合は画面の公開へ進まず、差分を調べて再同期・再検証してください。`gcloud storage rsync` は既定では宛先だけにある余分なオブジェクトを削除しないため、生成物と配信先を一致させる場合は上記の削除フラグを使います。

## フロントエンドの公開と画面確認

データを配置・検証した後、フロントエンドを `main` に反映します。純資産分布の公開データとフロントエンドはどちらも schema version 3 を使用します。`.github/workflows/deploy-sveltekit.yml` は `main` への push で GitHub Pages をビルド・公開します。workflow が参照する GitHub リポジトリ変数 `VITE_STORAGE_BASE_URL` が正しい配信元を指すことも確認してください。workflow は GCS へデータをアップロードしません。

Actions の deploy job が成功したら公開ページを再読み込みし、次を確認します。

- 初期選択でグラフが表示される。
- 属性を変更すると対応する系列が表示される。
- 複数系列比較で各系列の値が表示される。
- 開いたままにしていたページは再読み込みして最新のデータを読み込む。

## データ prefix と公開範囲

同期コマンドはデータごとの prefix に限定してください。バケットルートに対して `--delete-unmatched-destination-objects` を付けると、公開対象外のデータまで削除するおそれがあります。`gcloud storage rm` を使って prefix 全体を削除する必要はありません。削除が必要な場合は、対象 prefix とオブジェクト一覧を確認してから、そのデータの公開範囲に限定して実行します。
