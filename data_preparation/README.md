# データの取得・加工

このディレクトリには、e-Stat の統計データと国土交通省の地理データを取得・加工する Rust プログラムがあります。以下のコマンドはすべて `data_preparation/` から実行します。

[`inspections/`](inspections/README.md) は、最終的にどのデータをどのように加工するかを検討するための実験・調査用フォルダです。データ候補の比較、出典や値の解釈の確認、加工方法の試行とその記録を置きます。

生成した配信用データを Cloud Storage に配置し、GitHub Pages のフロントエンドを公開する手順は[配信データの公開手順](PUBLISHING.md)を参照してください。公開対象と GCS の prefix をデータ別に指定し、取得時の API 応答や監査用ファイルを誤って同期しない手順を記載しています。

## 必要な環境

- Rust と Cargo
- e-Stat のデータを取得する場合: e-Stat API のアプリケーション ID
- 都道府県データを更新する場合: Node.js、npm、`unzip`
- 野菜の公式ワークブックを再取得・照合する場合: ネット接続と `unzip`

都道府県データの簡略化には npm パッケージの Mapshaper を使います。初回実行前に依存パッケージをインストールしてください。

```sh
npm ci
```

## 小売物価統計データ

e-Stat のアプリケーション ID を環境変数に設定して実行します。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- commodity-prices
```

メタデータ、API の生データ、加工後のデータは `output/commodity_prices/` 以下に保存されます。取得済みファイルの再利用条件は各処理で確認されます。

## 年齢階級・雇用形態・教育・性別別の雇用者数

e-Stat の表 ID `0003006608` から選択肢と系列データを生成します。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- workers-by-age-employment-type-education-sex
```

メタデータ、ページ分割した取得データ、フロントエンド向け JSON は `output/workers_by_age_employment_type_education_sex/` 以下に保存されます。取得済みの API 応答を使って再変換するときは `cargo run -- workers-by-age-employment-type-education-sex --transform-only` を実行します。変換だけの再実行では取得済みの原本を保持します。生成に失敗した場合は原因を解消して再実行してください。コマンドが成功した生成結果だけを公開してください。

## 属性別純資産額階級別世帯数

e-Stat 表 `0004056120` の全メタデータ・ページ分割データを取得し、分類辞書、属性組み合わせ、通常の純資産額階級別世帯数を生成します。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- asset-distribution
```

取得済みのAPI応答から再変換する場合は `cargo run -- asset-distribution --transform-only` を実行します。生データは `output/asset_distribution/` に保存され、変換だけの再実行では保持されます。配信用データは同ディレクトリの `manifest.json` と `items/` 配下のJSONチャンクです。schema version 3 の manifest は分類辞書と表示階級順を保持し、属性の直積インデックスと各チャンクの連続範囲から組み合わせ・値ファイルの対応を導出します。manifest と各チャンクは version 3 で、チャンクは21階級の値を収録します。表示階級は再掲 `R01`～`R03` と通常階級 `02`～`19` で、再掲 `01` は重複計上を避けるため含めません。フロントエンドは version 3 のみを受け付けます。version 2 および旧形式との後方互換性、変換やフォールバックはありません。追試用サンプルは `inspections/asset_distribution/` に保存されます。表の分類・値の解釈、出力サイズと転送量の区別は[調査記録](inspections/asset_distribution/README.md)を参照してください。生成に失敗した場合は原因を解消して再実行してください。成功した生成結果だけを公開してください。

このデータは `VITE_STORAGE_BASE_URL` で指定する外部ストレージの `/asset_distribution/` 以下から配信します。GitHub Actions の現行 workflow はフロントエンドのみを GitHub Pages に公開し、外部データのアップロードやキャッシュ更新は行いません。

## 野菜の産地別卸売データ

clone 後、ネット接続と e-Stat のアプリケーション ID を用意し、`data_preparation/` から次を実行すると野菜の産地リレー用データを生成できます。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- vegetable-relay
```

生成の仕組み、調査コマンド、ファイル形式、データの解釈は[野菜リレーの実験・調査記録](inspections/vegetable_production_relay/README.md)を参照してください。

## 都道府県データ

```sh
cargo run -- prefectures-update
```

国土交通省の国土数値情報から N03（行政区域）データを取得し、都道府県ごとに簡略化した GeoJSON を作成します。ZIP と展開した元データは `input/prefectures/N03-2026/`、出力は `output/prefectures/N03-20260101_prefecture_simplified.geojson` に保存されます。既存の ZIP と元データは再利用します。

## ヘルプ

```sh
cargo run -- --help
```

## 参考リンク

- [e-Stat API 仕様（Swagger UI）](https://api.e-stat.go.jp/swagger-ui/e-statapi3.0.html)
- [e-Stat API 利用ガイド](https://www.e-stat.go.jp/api/api-info/e-stat-manual3-0)
- [小売物価統計データ（e-Stat）](https://www.e-stat.go.jp/dbview?sid=0003421913)
- [国土数値情報ダウンロードサイト](https://nlftp.mlit.go.jp/ksj/)
