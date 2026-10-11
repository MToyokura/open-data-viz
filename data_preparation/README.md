# データの取得・加工

このディレクトリには、統計データや地理データを取得・加工する Rust プログラムがあります。実行するコマンドを指定して処理を選びます。

## 必要な環境

- Rust と Cargo
- 野菜データのワークブック検査を行う場合: `unzip`
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

2021～2023 年の野菜50品目について、品目別に e-Stat のメタデータと観測値を保存し、オフラインで再検証できるようにしています。月を示す明示的な軸がない表の13個の `cat03` ブロックを、品目ごとの公式「計」・各月 Excel 表と全セル照合しました。各品目の39ワークブックを対象に、1,152セルずつ照合しています。すべての品目で3年分の月対応と数量・価格単位を確認済みです。そらまめの5期間は複数のブロックが全比較セルで完全一致し、月ごとのブロック対応を特定できませんが、候補間で表示値は同一です。この対応上の曖昧さは照合記録に記録しています。

照合の根拠、表 ID、元ファイルのチェックサム、再検証方法は [野菜データの調査記録](inspections/vegetable_production_relay/README.md) と [2021～2023 年の月対応の根拠](inspections/vegetable_production_relay/2021_2023_temporal_evidence.md) にあります。表示対象は 2021～2023 年に限定します。

月対応と単位は各品目の公式ワークブックで確認済みです。解釈の根拠と残る年次比較の確認事項は [野菜データの解釈メモ](inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md) にあります。Rust の `vegetable_production_relay::aggregation` は、品目別に検証済みの正規化データを集計し、主要消費地域9区分の数量を産地都道府県・月ごとに出力します。表示単位未満、該当なし、不詳・調査なしの値を数値合計に含めず、状態別の件数を保持します。品目のテーブル ID は品目ごとの temporal mapping から読み込みます。

品目ごとのワークブック照合と月対応 mapping がリポジトリに用意されている状態で、次のコマンドを実行します。このコマンドはワークブック検査や mapping の作成は行いません。metadata と e-Stat 観測値は、有効な capture があれば再利用し、なければ API から取得して検証します。その後、全品目を正規化・年次集計し、`products.json` を更新します。新しい環境で実行する場合も `ESTAT_APP_ID` が必要です。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- vegetable-relay
```

<!-- // here -->
各品目の `{品目キー}.summary.json` は地図用の集計 summary（schema v2）です。既存の `{品目キー}.json`（schema v1）も移行期間中は生成し、旧フロントエンドを維持します。`{品目キー}/details/{産地コード}.json` には産地別36か月分の内訳を保存し、選択時のみ取得します。従来の集計 JSON に含まれた検証ハッシュと元セルの位置は `{品目キー}/audit.json` に保持します。品目一覧の生成時に summary と47個の detail の整合性を検証し、不完全な品目は公開しません。従来の再利用判定で schema v1 が再利用された場合でも、`products.json` の生成処理が v1 から v2 の summary・detail・audit を構築して照合します。移行前の旧 JSON を別の場所に退避しておき、summary と detail が揃っていることを確認してください。公開する際は detail・audit・summary を先に配信し、その後 `products.json` を更新してください。
metadata と観測値の capture は `inspections/vegetable_production_relay/runs/` に保存されます。通常の `vegetable-relay` 実行では、これらの capture 参照を inspection 用 inventory に書き戻しません。同じ品目の出力がすでにあり、mapping・表示名・正規化済み入力が一致すれば再利用し、いずれかが異なれば再生成します。アプリは `products.json` から品目を選び、対応する品目 JSON（schema_version 2 の軽量 summary）を読み込みます。ローカル開発では、リポジトリルートの `npm run serve` が `data_preparation/output/` を配信します。年ごとの単位を表示し、不完全なセルの状態を示します。令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。この表への影響は未確認のため、年をまたぐ比較にはご注意ください。

#### 50 品目の一括処理

以下は月対応の根拠を再取得・再検証するための inspection 手順です。通常のデータ生成とは独立しており、検証済み mapping がリポジトリにあれば `vegetable-relay` の前に実行する必要はありません。ワークブックの取得にはネット接続が必要です。metadata と観測値の取得には `ESTAT_APP_ID` が必要です。

```sh
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- vegetable-relay-inspect-workbooks
cargo run -- vegetable-relay-acquire-product-observations
cargo run -- vegetable-relay-verify-product-mappings
```

3つ目のコマンドは取得済みワークブックと観測値を使って mapping を検証し、inspection 用 inventory を更新します。通常の生成では `cargo run -- vegetable-relay` を実行します。metadata・観測値の取得には同じ capture/replay 処理を使い、有効な capture を再利用します。mapping・表示名・正規化済み入力が既存 JSON と一致する品目も再利用します。必要な検証や生成に失敗すると処理全体がエラー終了し、`products.json` は更新されません。これらの操作は Rust の CLI から実行できます。

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
