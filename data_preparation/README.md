# データの取得・加工

このディレクトリには、e-Stat の統計データと国土交通省の地理データを取得・加工する Rust プログラムがあります。以下のコマンドはすべて `data_preparation/` から実行します。

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

## 野菜の産地別卸売データ

clone 後、ネット接続と e-Stat のアプリケーション ID を用意し、`data_preparation/` から次を実行すると野菜の産地リレー用データを生成できます。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- vegetable-relay
```

生成の仕組み、調査コマンド、ファイル形式、データの解釈は[野菜リレー調査記録](inspections/vegetable_production_relay/README.md)を参照してください。

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
