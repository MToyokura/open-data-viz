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

### 野菜の産地別卸売データ

2021～2023 年の野菜50品目について、品目別に e-Stat のメタデータと観測値を保存し、オフラインで再検証できるようにしています。月を示す明示的な軸がない表の13個の `cat03` ブロックを、品目ごとの公式「計」・各月 Excel 表と全セル照合しました。各品目の39ワークブックを対象に、1,152セルずつ照合しています。すべての品目で3年分の月対応と数量・価格単位を確認済みです。そらまめは5期間で複数のブロックが全比較セルで完全一致したため、候補と表示値が同等である旨をデータと画面に記録しています。

照合の根拠、表 ID、元ファイルのチェックサム、再検証方法は [野菜データの調査記録](inspections/vegetable_production_relay/README.md) と [2021～2023 年の月対応の根拠](inspections/vegetable_production_relay/2021_2023_temporal_evidence.md) にあります。表示対象は 2021～2023 年に限定します。

月対応と単位は各品目の公式ワークブックで確認済みです。解釈の根拠と残る年次比較の確認事項は [野菜データの解釈メモ](inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md) にあります。Rust の `vegetable_production_relay::aggregation` は、品目別に検証済みの正規化データを集計し、主要消費地域9区分の数量を産地都道府県・月ごとに出力します。表示単位未満、該当なし、不詳・調査なしの値を数値合計に含めず、状態別の件数を保持します。品目のテーブル ID は品目ごとの temporal mapping から読み込みます。

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
cargo run -- vegetable-relay-product-index
```

別品目を集計する場合は、その品目専用の mapping と正規化出力を使い、出力先の後ろに品目キーと表示名を指定します（例: `... <output-dir> veg-12 キャベツ`）。mapping に記録された各年のテーブル ID を使って検証し、`veg-12.json` のように保存します。品目 JSON を追加した後で `cargo run -- vegetable-relay-product-index` を実行すると、検証済みファイルだけを `products.json` に登録します。アプリはこの一覧から品目選択肢を作り、選択した品目の JSON を読み込みます。ローカル開発では、リポジトリルートの `npm run serve` が `data_preparation/output/` を配信します。年ごとの単位を表示し、不完全なセルの状態を示します。令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。この表への影響は未確認のため、年をまたぐ比較にはご注意ください。

### 50 品目の一括処理

全品目の source inventory と metadata capture が揃ったら、次の順に実行します。ワークブックの取得にはネット接続が必要です。e-Stat API 観測値の取得には `ESTAT_APP_ID` が必要です。月対応の検証に失敗した品目は JSON を生成しません。

```sh
python3 inspections/vegetable_production_relay/acquire_product_workbooks.py
python3 inspections/vegetable_production_relay/acquire_product_observations.py
python3 inspections/vegetable_production_relay/verify_product_temporal_mappings.py
python3 inspections/vegetable_production_relay/generate_product_data.py
```

各品目の 39 ワークブックを API の 13 `cat03` ブロックすべてと 1,152 セルで照合し、全期間の対応が成立することを確認します。同じ値を持つ候補ブロックが複数ある場合は等価候補として記録し、その値の同一性を確認します。照合マッピング、ワークブックの URL と SHA-256 は `inspections/vegetable_production_relay/` に記録します。元ワークブック、metadata、観測値のキャッシュは `runs/` 以下の Git-ignore 対象に保存します。

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
