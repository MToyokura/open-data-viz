# 配信データの公開

`data_preparation/output/` で生成したブラウザー用データを Google Cloud Storage に公開します。API 原本、取得時メタデータ、監査用ファイルは対象に含めません。

## 公開対象

| データ | 公開ファイル |
| --- | --- |
| 純資産分布 | `asset_distribution/manifest.json`、`asset_distribution/items/*.json` |
| 小売物価 | `commodity_prices/meta_data/{cat02_lookup,area_lookup}.json`、`commodity_prices/transformed_data/` 内の JSON |
| 雇用者数 | `workers_by_age_employment_type_education_sex/meta_data/options.json`、`transformed_data/` 内の JSON |
| 野菜の産地リレー | `vegetable_production_relay/products.json`、`*.summary.json`、`veg-*/details/` 内の JSON |
| 都道府県地図 | `prefectures/N03-20260101_prefecture_simplified.geojson` |

公開対象は `scripts/publish.sh` に列挙しています。生成形式やフロントエンドの読み込み先を変更した場合は、この一覧とスクリプトも更新してください。

## 認証と公開

GCS への書き込み権限を持つアカウントで `gcloud` 認証を済ませてください。リポジトリルートの `.env` または環境変数に公開先を設定します。`.env` は Git 管理対象外です。公開先はバケットまたは公開データ専用の prefix にしてください。

```sh
GCS_BUCKET=gs://YOUR_BUCKET
```

生成済みデータを確認してから、リポジトリルートで一つのコマンドを実行します。

```sh
./data_preparation/scripts/publish.sh
```

特定のデータだけを公開する場合は `--only` にプロジェクト名をカンマ区切りで指定します。たとえば、純資産分布と雇用者数を公開する場合は次のように実行します。

```sh
./data_preparation/scripts/publish.sh --only asset_distribution,workers_by_age_employment_type_education_sex
```

指定しなかったプロジェクトの出力はアップロード対象に含まれません。プロジェクト名はスクリプトの `--help` で確認できます。

スクリプトは公開対象だけを一時ディレクトリへ集め、JSON / GeoJSON を gzip 圧縮して元と同じ URL・ファイル名でアップロードします。各オブジェクトには適切な Content-Type、`Content-Encoding: gzip`、共通の短い Cache-Control を設定します。アップロードは対象ファイルごとに行われ、途中で失敗した場合はエラー終了します。一部のファイルだけ更新されることがあるため、原因を修正して同じコマンドを再実行してください。

## 公開後の確認

代表ページを開き、純資産分布の初期表示と条件変更、小売物価・雇用者数の系列表示、野菜リレーの一覧と詳細、地図表示を確認します。ブラウザーの開発者ツールで JSON が読み込めることと、応答の Content-Type / Content-Encoding を確認してください。データ公開後に画面を再読み込みして最新データを取得します。GitHub Pages のフロントエンド公開は GCS データ公開とは別です。
