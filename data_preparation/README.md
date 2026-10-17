# データの取得・加工

このディレクトリには、e-Stat の統計データと国土交通省の地理データを取得・加工する Rust プログラムがあります。以下のコマンドはすべて `data_preparation/` から実行します。

[`inspections/`](inspections/README.md) は、最終的にどのデータをどのように加工するかを検討するための実験・調査用フォルダです。データ候補の比較、出典や値の解釈の確認、加工方法の試行とその記録を置きます。

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

メタデータ、ページ分割した取得データ、フロントエンド向け JSON は `output/workers_by_age_employment_type_education_sex/` 以下に保存されます。取得済みの API 応答を使って再変換するときは `cargo run -- workers-by-age-employment-type-education-sex --transform-only` を実行します。

## 属性別純資産額階級別世帯数

e-Stat 表 `0004056120` の全メタデータ・ページ分割データを取得し、分類辞書、属性組み合わせ、通常の純資産額階級別世帯数を生成します。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- asset-distribution
```

取得済みのAPI応答から再変換する場合は `cargo run -- asset-distribution --transform-only` を実行します。生データは `output/asset_distribution/` に保存されます。配信用データは同ディレクトリの `manifest.json` と `items/` 配下のJSONチャンクで、manifestに分類辞書・組み合わせ・値ファイルの対応を、チャンクに通常階級の値を収録します。旧単一ファイル `asset_distribution.json` は生成しません。追試用サンプルは `inspections/asset_distribution/` に保存されます。表の分類・値の解釈、現行出力の規模、転送量の計測方法は[調査記録](inspections/asset_distribution/README.md)を参照してください。

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
