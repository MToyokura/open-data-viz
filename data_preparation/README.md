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

だいこんの e-Stat データについて、2021～2023 年の候補表のメタデータと観測値を保存し、オフラインで再検証できるようにしています。2021～2023 年の表では、月を示す明示的な軸がありませんが、e-Stat が別途公開する「計」と各月の Excel 表を全セル照合し、13 個の `cat03` ブロックを特定しました。先頭ブロックが年間の「計」、残る 12 ブロックが 1～12 月です。各 Excel 表は対応するブロックの 1,152 セルすべてと一致しています。

照合の根拠、表 ID、元ファイルのチェックサム、再検証方法は [野菜データの調査記録](inspections/vegetable_production_relay/README.md) と [2021～2023 年の月対応の根拠](inspections/vegetable_production_relay/2021_2023_temporal_evidence.md) にあります。表示対象は 2021～2023 年に限定します。

月対応と単位は公式ワークブックで確認済みです。解釈の根拠と残る年次比較の確認事項は [野菜データの解釈メモ](inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md) にあります。Rust の `vegetable_production_relay::aggregation` は正規化データを集計し、主要消費地域9区分の数量を産地都道府県・月ごとに出力します。表示単位未満、該当なし、不詳・調査なしの値を数値合計に含めず、状態別の件数を保持します。出力先ディレクトリには品目ごとのファイル（現在は `daikon.json`）を作成します。

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

集計コマンドは既定で `output/vegetable_production_relay/daikon.json` に出力します。必要なら最後に出力先ディレクトリを指定できます。アプリは `VITE_STORAGE_BASE_URL/vegetable_production_relay/daikon.json` からデータを読み込みます。ローカル開発では、リポジトリルートの `npm run serve` が `data_preparation/output/` を配信します。年ごとの単位を表示し、不完全なセルの状態を示します。令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。この表への影響は未確認のため、年をまたぐ比較にはご注意ください。

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
