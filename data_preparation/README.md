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

### 野菜の産地別卸売データ（調査中）

だいこんの e-Stat データについて、2020～2024 年の候補表のメタデータと観測値を保存し、オフラインで再検証できるようにしています。2021～2023 年の表では、月を示す明示的な軸がありませんが、e-Stat が別途公開する「計」と各月の Excel 表を全セル照合し、13 個の `cat03` ブロックを特定しました。先頭ブロックが年間の「計」、残る 12 ブロックが 1～12 月です。各 Excel 表は対応するブロックの 1,152 セルすべてと一致しています。

照合の根拠、表 ID、元ファイルのチェックサム、再検証方法は [野菜データの調査記録](inspections/vegetable_production_relay/README.md) と [2021～2023 年の月対応の根拠](inspections/vegetable_production_relay/2021_2023_temporal_evidence.md) にあります。2024 年の分類と観測範囲は [2024 年の検証記録](inspections/vegetable_production_relay/2024_daikon_validation.md) を参照してください。

月対応は公式表の全セル照合で確認できましたが、独立レビューは未完了です。また、出荷元・消費地域の意味、2021年の単位、欠測値、年をまたぐ比較可能性、集計値の独立照合の検証が残っています。Rust の `vegetable_production_relay::aggregation` は M5-B 出力を集計し、各地域の元値・行番号を残す確認用データを生成します。出力先ディレクトリには品目ごとのファイル（現在は `daikon.json`）を作成します。生成物とフロントエンドの表示はレビュー待ちであり、公開統計として利用できません。

まず 2021～2023 年ごとに [M5-B 正規化手順](inspections/vegetable_production_relay/M5A_NORMALIZATION.md) を実行し、その3つの新しい正規化出力を `data_preparation/` から集計コマンドへ渡します。正規化先ディレクトリを先に作成し、集計結果の出力先にはディレクトリを指定してください。同じ品目の JSON がすでにある場合は上書きせずエラーになります。

```sh
cargo run -- vegetable-relay-normalize-2021-2023 \
  <observations-run-dir-2021> <metadata-run-dir-2021> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  <new-m5b-output-dir-2021>
cargo run -- vegetable-relay-normalize-2021-2023 \
  <observations-run-dir-2022> <metadata-run-dir-2022> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  <new-m5b-output-dir-2022>
cargo run -- vegetable-relay-normalize-2021-2023 \
  <observations-run-dir-2023> <metadata-run-dir-2023> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  <new-m5b-output-dir-2023>
cargo run -- vegetable-relay-aggregate-2021-2023 \
  <new-m5b-output-dir-2021>/m5b_inspection_only.json \
  <new-m5b-output-dir-2022>/m5b_inspection_only.json \
  <new-m5b-output-dir-2023>/m5b_inspection_only.json \
  inspections/vegetable_production_relay/runs \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json
```

集計コマンドは既定で `output/vegetable_production_relay/daikon.json` に出力します。必要なら最後に出力先ディレクトリを指定できます。アプリは `VITE_STORAGE_BASE_URL/vegetable_production_relay/daikon.json` からこの確認用データを読み込みます。ローカル開発では、リポジトリルートの `npm run serve` が `data_preparation/output/` を配信します。年ごとの単位を表示し、欠測・意味未確定のセルを合計に含めず、各集計の9地域分の出典位置を確認できます。

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
