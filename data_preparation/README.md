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

e-Stat API を使う場合は、リポジトリルートの `.env` にアプリケーション ID を設定します。`.env` は Git 管理対象外です。`scripts/prepare.sh` はこのファイルを読み込み、引数をそのまま Cargo に渡します。既にシェルで `ESTAT_APP_ID` を設定している場合は、その値が優先されます。以降のコマンド例は `data_preparation/` から実行します。

```sh
# リポジトリルートの .env
ESTAT_APP_ID="取得したアプリケーションID"
```

## 小売物価統計データ

e-Stat 表 `0003421913` からメタデータと全商品の観測値を取得し、検証済み capture とブラウザー用データを生成します。初回取得・更新には e-Stat API のアプリケーション ID が必要です。コマンドは `prepare.sh` 経由で実行します。

```sh
./scripts/prepare.sh run -- commodity-prices
```

取得済みの検証済み応答からオフラインで再生成する場合:

```sh
./scripts/prepare.sh run -- commodity-prices --transform-only
```

capture は `output/commodity_prices/raw_data/` と `meta_data/acquisition.json` に保存され、各応答ページの件数・チェックサムが記録されます。変換だけの再実行には capture が必要ですが、API ID やネットワーク接続は不要です。生成物は `output/commodity_prices/v2/` に保存されます。`v2/meta_data/` には商品・地域 lookup、`v2/transformed_data/` には地域ごとに商品 lookup 順の連続10商品をまとめた JSON チャンクが含まれます。生成・検証は staging 上で行われ、失敗時は直前の有効 bundle が保持されます。allowlist のローカル確認は `./data_preparation/scripts/publish.sh --only commodity_prices --dry-run`、公開と手動ロールバック手順は[配信データの公開手順](PUBLISHING.md)を参照してください。

## 年齢階級・雇用形態・教育・性別別の雇用者数

e-Stat の表 ID `0003006608` から選択肢と系列データを生成します。

```sh
./scripts/prepare.sh run -- workers-by-age-employment-type-education-sex
```

メタデータ、ページ分割した取得データ、フロントエンド向け JSON は `output/workers_by_age_employment_type_education_sex/` 以下に保存されます。取得済みの API 応答を使って再変換するときは `./scripts/prepare.sh run -- workers-by-age-employment-type-education-sex --transform-only` を実行します。変換だけの再実行では取得済みの原本を保持します。生成に失敗した場合は原因を解消して再実行してください。コマンドが成功した生成結果だけを公開してください。

## 属性別純資産額階級別世帯数

e-Stat 表 `0004056120` の全メタデータ・ページ分割データを取得し、分類辞書、属性組み合わせ、通常の純資産額階級別世帯数を生成します。

```sh
./scripts/prepare.sh run -- asset-distribution
```

取得済みのAPI応答から再変換する場合は `./scripts/prepare.sh run -- asset-distribution --transform-only` を実行します。生データは `output/asset_distribution/` に保存され、変換だけの再実行では保持されます。配信用データは同ディレクトリの `manifest.json` と `items/` 配下のJSONチャンクです。schema version 3 の manifest は分類辞書と表示階級順を保持し、属性の直積インデックスと各チャンクの連続範囲から組み合わせ・値ファイルの対応を導出します。manifest と各チャンクは version 3 で、チャンクは21階級の値を収録します。表示階級は再掲 `R01`～`R03` と通常階級 `02`～`19` で、再掲 `01` は重複計上を避けるため含めません。フロントエンドは version 3 のみを受け付けます。version 2 および旧形式との後方互換性、変換やフォールバックはありません。追試用サンプルは `inspections/asset_distribution/` に保存されます。表の分類・値の解釈、出力サイズと転送量の区別は[調査記録](inspections/asset_distribution/README.md)を参照してください。生成に失敗した場合は原因を解消して再実行してください。成功した生成結果だけを公開してください。

このデータは `VITE_STORAGE_BASE_URL` で指定する外部ストレージの `/asset_distribution/` 以下から配信します。GitHub Actions の現行 workflow はフロントエンドのみを GitHub Pages に公開し、外部データのアップロードやキャッシュ更新は行いません。

## 消費者物価指数のAPI調査

e-Stat 表 `0004052037` のメタデータと小さなAPI応答標本を取得します。

```sh
./scripts/prepare.sh run -- consumer-price-index-inspect
```

分類規模、取得ファイル、API接続確認の状況は[調査記録](inspections/consumer_price_index/README.md)を参照してください。

## 消費者物価指数データ

e-Stat 表 `0004052037` から指数（`tab=1`）の月次データ（`lvTime=4`）を取得し、項目・地域・月のルックアップと系列チャンクを生成します。初回生成と更新時は e-Stat API のアプリケーション ID が必要です。

```sh
./scripts/prepare.sh run -- consumer-price-index
```

変換処理を直した後、取得済みのAPI応答から再生成する場合:

```sh
./scripts/prepare.sh run -- consumer-price-index --transform-only
```

出力は `output/consumer_price_index/` 以下です。`meta_data/` に取得条件と件数、`raw_data/` にメタデータとページ境界を検証したAPI応答、`transformed_data/` に配信用データを保存します。取得・変換は `.staging/` 内で完了と検証を済ませてから有効データへ切り替え、失敗時は直前の有効データを保持します。変換データは `items.json`、`regions.json`、`months.json`、`manifest.json` と `series/` 配下の地域別・連続10項目単位のチャンクで構成されます。各系列は `itemCode`、`areaCode`、昇順の `[YYYY-MM, value]` 配列だけを持ち、欠損月は省略します。応答の `NOTE` が「該当データがない」と定義する記号値も欠損として省略し、その他の非数値はエラーにします。指数以外の測定値と年次値は出力しません。保存済み代表系列との不一致は、過去値が改定された可能性として警告します。調査時に確認したAPIメタデータ総数とデータ応答総数の差は、取得処理では `getStatsData` の `TOTAL_NUMBER` と連続した `NEXT_KEY` を基準に検証します。

## 野菜の産地別卸売データ

clone 後、ネット接続と e-Stat のアプリケーション ID を用意し、`data_preparation/` から次を実行すると野菜の産地リレー用データを生成できます。

```sh
./scripts/prepare.sh run -- vegetable-relay
```

取得済みの検証済みcaptureは再利用します。配信用データは実行ごとに原本とmappingから再生成されます。生成に失敗した場合は出力が不完全な状態になり得るため、原因を修正して再実行してください。

生成の仕組み、調査コマンド、ファイル形式、データの解釈は[野菜リレーの実験・調査記録](inspections/vegetable_production_relay/README.md)を参照してください。

## 海面漁業魚種別漁獲量

全国累年統計のワークブックから年別・魚種別漁獲量と原資料の注記を生成します。保存済みワークブックを再変換する場合は `./data_preparation/scripts/prepare.sh run -- marine-fishery-catch --transform-only` を実行します。調査時の構造と照合点は[調査記録](inspections/marine_fishery_catch/README.md)を参照してください。

## 都道府県データ

```sh
./scripts/prepare.sh run -- prefectures-update
```

国土交通省の国土数値情報から N03（行政区域）データを取得し、都道府県ごとに簡略化した GeoJSON を作成します。ZIP と展開した元データは `input/prefectures/N03-2026/`、出力は `output/prefectures/N03-20260101_prefecture_simplified.geojson` に保存されます。既存の ZIP と元データは再利用します。

## ヘルプ

```sh
./scripts/prepare.sh run -- --help
```

## 参考リンク

- [e-Stat API 仕様（Swagger UI）](https://api.e-stat.go.jp/swagger-ui/e-statapi3.0.html)
- [e-Stat API 利用ガイド](https://www.e-stat.go.jp/api/api-info/e-stat-manual3-0)
- [小売物価統計データ（e-Stat）](https://www.e-stat.go.jp/dbview?sid=0003421913)
- [国土数値情報ダウンロードサイト](https://nlftp.mlit.go.jp/ksj/)
