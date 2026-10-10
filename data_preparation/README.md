# データの取得・加工

このディレクトリには、統計データや地理データを取得・加工する Rust プログラムがあります。実行するコマンドを指定して処理を選びます。

## 必要な環境

- Rust と Cargo
- 都道府県データを更新する場合: Node.js 20.11 以降、npm、`unzip`

都道府県データの簡略化には npm パッケージの Mapshaper を使います。初回実行前に、このディレクトリで依存パッケージをインストールしてください。

```sh
npm ci
```

## 実行方法

以下のコマンドは、この `data_preparation` ディレクトリで実行します。

### 小売物価統計データ

e-Stat API を使うため、実行前に e-Stat のアプリケーション ID を `ESTAT_APP_ID` 環境変数に設定してください。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- commodity-prices
```

取得したメタデータ、API の生データ、加工後のデータは、それぞれ `output/commodity_prices/meta_data/`、`output/commodity_prices/raw_data/`、`output/commodity_prices/transformed_data/` に保存されます。既存のメタデータや生データのファイルがある場合、そのファイルを再利用して取得を省略します。

### 都道府県データ

```sh
cargo run -- prefectures-update
```

国土交通省の国土数値情報から N03（行政区域）データを取得し、都道府県単位に簡略化した GeoJSON を作成します。ZIP と展開した元データは `input/prefectures/N03-2026/` に、生成したファイルは `output/prefectures/N03-20260101_prefecture_simplified.geojson` に保存されます。ZIP や展開済みの元データがすでにある場合は再利用します。

### ヘルプ

引数を省略するか、`--help` を指定すると利用できるコマンドを表示します。

```sh
cargo run -- --help
```

## 参考リンク

- [e-Stat API 仕様（Swagger UI）](https://api.e-stat.go.jp/swagger-ui/e-statapi3.0.html)
- [e-Stat API 利用ガイド](https://www.e-stat.go.jp/api/api-info/e-stat-manual3-0)
- [小売物価統計データ（e-Stat）](https://www.e-stat.go.jp/dbview?sid=0003421913)
- [国土数値情報ダウンロードサイト](https://nlftp.mlit.go.jp/ksj/)
