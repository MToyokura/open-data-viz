# 純資産分布 manifest の組み合わせ情報を小さくする計画

## 目的

純資産分布の manifest にある組み合わせ情報の重複をなくし、未圧縮の JSON 自体を小さくする。圧縮転送の設定とは分けて評価する。現在の Google Cloud Storage 応答は `Content-Encoding` がなく `identity` で、manifest 全体の 2,191,115 bytes がそのまま返っている。

schema version 3 で既存形式を全面的に置き換える。schema version 2 や旧データとの後方互換性、移行期間の併存、旧データの保持、旧形式への切り戻しは要件に含めない。公開先のパスも分けず、既存の配信場所に新形式を配置する。

現在の manifest を gzip 圧縮するとローカル実測で72,558 bytesになる。これは有効な配信改善だが、配信 JSON の構造を小さくする変更とは別に扱う。

## 現状の計測と構造

2026-10-17 に生成した schema version 2 の `output/asset_distribution/manifest.json` を計測した。

| 項目 | 容量 |
| --- | ---: |
| manifest 全体（整形 JSON） | 2,191,115 bytes |
| `combinations`（整形 JSON） | 1,690,796 bytes |
| `chunks`（整形 JSON） | 309,084 bytes |
| `dimensions`（整形 JSON） | 12,217 bytes |
| `metadata`（整形 JSON） | 343 bytes |
| gzip 後の manifest | 72,558 bytes |

`combinations` は6,762件あり、全件 `published` である。件数は分類辞書の直積 `cat01=7 × cat02=6 × cat03=23 × cat05=7` と一致する。各項目は4軸分のコード、同じ情報から作れる `id`、`items_file`、全件共通の `status` と `reason: null` を持つ。

さらに `chunks[].combination_ids` にも同じ6,762件の完全な系列IDを重ねて記録している。現行の生成順でチャンクに割り当てられた組み合わせIDを確認したところ、各チャンクは組み合わせ一覧上の連続した範囲だった。このため、組み合わせ一覧とチャンク内のID一覧の両方を保存せず、分類辞書とチャンク範囲から情報を導出できる見込みがある。

## 変更対象と影響箇所

- 生成・ステージ検証: `data_preparation/src/modules/projects/asset_distribution.rs`
- manifest 検証、系列検索、チャンク読込: `frontend/src/routes/asset_distribution/utils/data.js`
- 配信URL: 既存の `VITE_STORAGE_BASE_URL` を使用し、純資産分布の manifest とチャンクは `/asset_distribution/` 以下から取得する。専用URL設定や workflow の変更は行わない。
- 属性選択と比較用の系列検索: `frontend/src/routes/asset_distribution/+page.svelte`
- 配信サイズの説明: `data_preparation/README.md`、`data_preparation/inspections/asset_distribution/README.md`
- 契約テストと画面テスト: `frontend/src/routes/asset_distribution/utils/__tests__/`、`frontend/e2e/asset-distribution.test.js`

## 表現案

### A. 直積とチャンク範囲から系列を導出する（第一候補）

manifest には属性軸の順序・区分順と系列総数を残し、6,762件の組み合わせ一覧を持たせない。各チャンクは `path`、`bytes`、`start_index`、`count` を持ち、直積上の連続範囲を示す。フロントエンドは `start_index` から `count` 件をデコードし、属性コード、現行形式の系列ID、チャンクパスを導出する。

例:

```json
{
  "schema_version": 3,
  "combination_index": {
    "dimension_ids": ["cat01", "cat02", "cat03", "cat05"],
    "count": 6762
  },
  "chunks": [
    { "path": "items/00000.json", "bytes": 259537, "start_index": 0, "count": 59 }
  ]
}
```

`dimension_ids` の配列順と各軸の `categories` の配列順から直積の順序を定義し、最後の軸を最速で変化させる。生成側とフロントエンドの両方で、`dimension_ids` が `dimensions` 内の全 `filter` 軸を重複なく一度ずつ含むことを検証する。未知の軸、`filter` 軸の欠落、`asset_class` 軸の混入を拒否し、`count` は対象軸の区分数の積と一致させる。チャンク範囲は重複・欠落なく `0..count` 全体を覆う必要がある。現行データ生成は全組み合わせ・全元分類の取得漏れをエラーにするため、部分公表状態を暗黙に作らず、欠けた系列は manifest の生成自体を失敗させる。`status` と `reason` は全組み合わせが公表済みである現行契約に沿って省略する。

チャンク内の値が持つ `combination_id` は現行文字列を維持し、生成された組み合わせIDと照合して検証できるようにする。チャンクファイルの値形式はこの変更の対象外とする。

### B. 短いコード表を持つ組み合わせ配列

任意の未公表組み合わせを将来も表現する必要がある場合は、各分類コードを区分配列の整数位置で表すタプルと、チャンク番号・状態コードだけを持つ配列を試作する。軸ID、長いID文字列、`items_file` の反復を避け、現行の `published` / `not_published` 表現を保てる。A案より実行時のデコード処理とフォーマットの解釈が増えるため、A案で必要な機能が満たせない場合の候補とする。

### C. 整形空白のみを削る案

JSON を minify してサイズも測るが、内容の繰り返しは残る。単独では十分な削減にならない可能性があるため、A・B案との比較用ベースラインに限る。

## 実装手順

1. A案の manifest を試作し、現行の metadata、区分順、6,762系列、124チャンク、142,002値を保ったまま生成する。直積インデックスとチャンク範囲の計算順は、現行 Rust の `cartesian_codes` と一致させる。
2. 生成側で、`dimension_ids` の重複・未知の軸・`filter` 軸の欠落・`asset_class` 軸の混入、カテゴリ区分の重複、直積数の不一致、範囲の重複・欠落・範囲外、チャンク容量超過、系列と値チャンクの不一致を拒否する。チャンクグループが連続範囲にならない場合は範囲形式を出さず、生成を失敗させる。
3. フロントエンドの型、manifest 検証、`findCombination`、比較候補の公表判定、チャンク取得を導出形式に置き換える。manifest 検証では生成側と同じ軸集合・直積数・範囲の条件を確認する。分類コードの順序と表示順は既存の metadata および `display_category_codes` から得る。
4. 形式を破壊的に置き換え、manifest とチャンクの `schema_version` を3にする。新フロントエンドは version 3 のみを受け付け、version 2、旧形式、旧形式からの変換・フォールバックを実装しない。旧形式との併存や旧形式への切り戻しは行わない。
5. 保存済み公式 API 応答から配信用データを再生成し、生成 bundle をステージ検証する。6,762件すべての系列ID、4軸コード、値ファイル対応が現行 bundle と一致することを一度検証する。この比較は新形式の内容確認だけに使い、旧形式を実行時に読み込んだり保持したりしない。検証済み bundle で、後述の公開手順に従って既存の配信ファイルを置き換える。
6. README と調査記録に新しい manifest 形式、旧形式との非互換性、旧形式から削除した処理、再計測値を記録する。
7. 既存の `VITE_STORAGE_BASE_URL` を配信元として使い、manifest とチャンクを `/asset_distribution/` 以下から取得する。純資産分布専用 URL、世代別 prefix、公開 workflow の環境変数追加は行わない。既存の配信場所の manifest とチャンクを v3 形式で置き換える。`Content-Encoding` の gzip 設定は別作業として扱い、v3 の未圧縮 JSON サイズと HTTP 転送量を別々に記録する。

## 公開手順と切り替え中の停止

フロントエンド、manifest、全チャンクの公開は同時には完了しない。最初のチャンクの上書き開始から、全ファイルの配置と新フロントエンドの公開後の確認が完了するまで、純資産分布の画面がバージョン不一致などで利用できなくなることを許容する。現時点で利用者はいないため、停止や切り替え完了の告知は不要とする。

1. ステージ検証済みの v3 bundle と、v3 のみを読み込むフロントエンドの公開用ビルドを準備する。
2. 既存の `/asset_distribution/items/` 配下に v3 の全チャンクを配置し、全件のアップロード完了を確認する。
3. 同じ配信場所の `manifest.json` を v3 に置き換える。
4. 配信先から manifest と参照先の全チャンクを取得する。すべてが version 3 であり、チャンクのバイト数、範囲から導出した系列ID、21階級の値が検証済み bundle と一致することを確認する。確認に失敗した場合は停止期間を継続し、配置を修正して再確認する。
5. v3 対応フロントエンドを公開する。公開先の画面を再読み込みし、初期選択、属性変更、複数系列比較で manifest とチャンクを取得でき、正常に表示されることを確認する。
6. 確認完了をもって切り替え完了とする。確認作業で切り替え前から旧画面を開いていた場合は、v3 データを読み込めないため、ページを再読み込みする。

## 検証と完了条件

- 6,762通りの組み合わせを直積インデックスから一意に再構成でき、IDは現在と一致する。
- 初期選択、属性変更、複数系列比較で同じ系列・同じ値チャンクが選ばれる。
- 各系列の21階級の表示順、観測値、`-`、表章単位未満の `0`、秘匿値の状態が保持される。
- 生成側とフロントエンドの異常系テストで、`dimension_ids` の重複・未知の軸・`filter` 軸の欠落・`asset_class` 軸の混入を拒否する。誤った軸集合に直積数とチャンク範囲を合わせたケースも拒否する。
- chunk range の検証が重複、欠落、境界ずれ、未参照チャンクを拒否する。version 2 および未知バージョンを拒否する。
- Rust の関連テスト、フロントエンドの unit tests、`npm run check`、`npm run build`、asset distribution の Playwright テストが通る。
- manifest の整形 JSON が250,000 bytes以下となり、現行2,191,115 bytesから約88%以上削減する。gzip 後のサイズも併記する。
- 既存の `/asset_distribution/` 配下に manifest とチャンクを新形式で配置し、純資産分布の画面で取得できることを確認する。共有 `VITE_STORAGE_BASE_URL` を使う他データの取得先は変わらないことを確認する。
- 公開手順に従い、配信先の manifest と全チャンクの検証、および新フロントエンド公開後の画面確認を完了する。

## 対象外

- gzip、Brotli、CDN、キャッシュ設定の変更。これらは JSON 構造の縮小と別に評価する。
- 値チャンク内の値形式や21階級表示の変更。
- schema version 2 との後方互換、旧 manifest を読み込むための移行処理、旧データの保持や切り戻し。
