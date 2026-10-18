# gzip 圧縮で配信データの送受信量を減らす計画

## 目的

データ準備で生成した JSON / GeoJSON を GCS へ送るときと、ブラウザーへ配信するときの転送量を gzip で減らす。配信ファイル名と JSON の形式は維持し、フロントエンドの `fetch()` と `response.json()` で通常どおり読み込める状態にする。

対象は `data_preparation/PUBLISHING.md` に列挙されている公開ファイル（純資産分布の manifest・チャンク、小売物価、雇用者数、野菜の産地リレー、都道府県 GeoJSON）。統計 API の取得処理や内部保存データは対象外。

## 現状

- 公開手順は `data_preparation/output/` から一時ディレクトリへファイルをコピーし、`gcloud storage rsync` で GCS に同期する。
- 公開対象のファイルは `.json` と `.geojson`。フロントエンドは通常の URL を `fetch()` して JSON として解析する。
- 純資産分布の計測記録では、GCS の manifest 応答に `Content-Encoding` がなく、未圧縮で配信された実績がある。ローカルで測った gzip サイズは実際の転送量ではない。
- フロントエンドは `VITE_STORAGE_BASE_URL` の GCS 公開 URL を参照する。ローカル開発ではルートの `http-server` が生成データをそのまま配信する。

## 推奨方式

公開用一時ディレクトリの各ファイルを、ファイル名と拡張子を保ったまま gzip 圧縮してから GCS に同期する。圧縮済みオブジェクトには次のメタデータを設定する。

- `Content-Type`: 圧縮前のファイル種別（JSON は `application/json`、GeoJSON は `application/geo+json`）。
- `Content-Encoding: gzip`: オブジェクト本体が gzip 圧縮されていることを示す。
- `Cache-Control`: 既存のキャッシュ方針を保ち、`no-transform` を含める。

GCS は gzip + `Content-Encoding: gzip` のオブジェクトを通常のダウンロード要求に対して自動展開することがある。その場合は保存サイズを減らせても、ブラウザーへの転送量は減らない。`Cache-Control: no-transform` を付け、GCS に圧縮済み表現をそのまま配信させる。ブラウザーは gzip 応答を HTTP レイヤーで自動展開するので、URL の `.gz` 化、独自の解凍処理、`fetch()` 呼び出し側での `Accept-Encoding` 指定は行わない。

現行の公開手順は `gcloud storage rsync` を使い、このコマンドの gzip オプションは `--gzip-in-flight`（アップロード時だけ圧縮し、GCS では未圧縮）である。したがって既存 rsync に単純に gzip-in-flight を足す方法は、ブラウザー向けの配信量削減にはならない。同期・削除の安全性を維持しつつ、圧縮済みステージを作り、rsync で元のファイル名のままアップロードし、上記メタデータを設定する方法を実装時に確定する。対象がすべて JSON / GeoJSON であることを前提に、同期対象 prefix を限定し、未圧縮オブジェクトを誤って残さない手順にする。

## 実装手順

1. `data_preparation/PUBLISHING.md` の公開コピー後に、公開対象ファイルのみから圧縮ステージを作る。gzip 出力は元の `.json` / `.geojson` 名を維持し、圧縮条件・小さいファイル・空ファイルの扱いは後述の条件に従う。
2. `gcloud storage rsync` のメタデータ指定で、圧縮済みステージを対象 prefix へアップロードする。`Content-Encoding`、圧縮前の `Content-Type`、`Cache-Control` を明示し、アップロードに削除オプションは付けず、削除は後述の許可一覧に基づく処理に分離する。純資産分布の manifest をアップロードする `gcloud storage cp` にも同じメタデータを指定する。既存 `sync_and_verify` と純資産分布のチャンク先行・manifest 後行の更新順を保つ。
3. 同期後は、転送が省略されたオブジェクトも含め、公開対象すべての object metadata を確認する。`rsync` のメタデータ指定だけで、内容に差分がないオブジェクトのメタデータも更新されるとは扱わない。不一致があれば、保存バイト列が圧縮ステージと一致することを確認したオブジェクトに限り、`gcloud storage objects update` で期待する `Content-Type`、`Content-Encoding: gzip`、既存のキャッシュ方針を含む `Cache-Control` を設定する。保存バイト列が一致しない場合は、圧縮ステージから当該オブジェクトを `gcloud storage cp` でメタデータとともに再アップロードする。修復対象は公開対象の明示的なオブジェクト一覧に限定し、修復後は内容とメタデータを再検証する。
4. 検証用ダウンロードは GCS 側の自動展開と gcloud 側の自動展開をそれぞれ避けて圧縮バイト列を取得し、圧縮ステージと比較する。GCS 側は `Cache-Control: no-transform` で防ぎ、メタデータ修復前の内容判定は後述の保存チェックサム比較で行う。gcloud 側は、既存 `sync_and_verify` と純資産分布のチャンク検証に使うダウンロード用 `gcloud storage rsync`、manifest 検証に使うダウンロード用 `gcloud storage cp` のすべてに `--do-not-decompress` を指定する。各検証では空の検証用ディレクトリへ取得し、gzip として展開できること、保存バイト列が圧縮ステージと一致すること、展開結果が元ファイルと一致することを確認する。併せて、すべての公開 JSON / GeoJSON に期待するメタデータがあることを確認する。純資産分布はチャンクの修復・検証が完了してから manifest を公開する。
5. `PUBLISHING.md` に、gzip ステージング、圧縮ファイルの同期、自動展開を無効にした検証、メタデータの修復、失敗時の停止条件を記録する。修復または再検証に失敗した場合は、後続の公開へ進まない。認証情報やバケット名は文書に追加しない。
6. 既存公開データを各 prefix 単位で再アップロードして圧縮メタデータへ移行する。配信設定変更後、ブラウザーの Network パネルで manifest、チャンク、代表的な各データの応答を確認する。

## 実装時に確定する同期・修復の詳細

以下は上記手順の具体的な実行条件とする。

### 圧縮条件と差分判定

GNU gzip の `gzip -n -6 -c` を使用し、元ファイル名とタイムスタンプを gzip ヘッダーに記録しない。実装・バージョンは実行環境で固定して記録する。0 バイトのファイルや JSON として解析できないファイルは公開前にエラーにする。`{}` や `[]` など有効な小さいファイルも、圧縮後にサイズが増える場合を含め一律 gzip にする。

アップロードの `rsync` は `--checksums-only` で差分を判定する。転送が省略された場合もメタデータ検証を行う。manifest も保存内容・メタデータが一致する場合は cp を省略する。

### Content-Type と同期範囲

現行の `PUBLISHING.md` では JSON と GeoJSON は別 prefix にある。次の対応で Content-Type を明示し、想定外の拡張子がステージにあれば停止する。

| 同期先 | Content-Type | 方法 |
| --- | --- | --- |
| `commodity_prices/` | `application/json` | rsync |
| `workers_by_age_employment_type_education_sex/` | `application/json` | rsync |
| `vegetable_production_relay/` | `application/json` | rsync |
| `prefectures/` | `application/geo+json` | rsync |
| `asset_distribution/items/` | `application/json` | rsync、削除なし |
| `asset_distribution/manifest.json` | `application/json` | チャンク検証後に cp |

将来同一 prefix に形式が混在する場合はアップロードを種別ごとに分け、各同期に削除オプションを付けない。manifest の cp にも Content-Type、Content-Encoding、Cache-Control を明示する。

### 削除と manifest の維持

削除はアップロードから分離する。現行 `PUBLISHING.md` の削除付き同期も、この条件に合わせて変更する。

- 公開対象の一覧と削除を許可する一覧を別々に作る。同期前に必須ファイル・参照先の存在と件数を確認し、空ステージや前回からの説明できない件数減少では停止する。
- チャンクの同期先は `asset_distribution/items/` に限定する。親の `asset_distribution/` をチャンクだけのステージと同期しない。既存 manifest はチャンク同期・検証中も維持する。
- gzip 移行中は旧チャンクを削除しない。チャンクの同期・修復・検証後に manifest を公開し、manifest 自体も検証する。将来の旧チャンク削除は、旧 manifest のキャッシュ有効期間と開いたままのページの利用期間を考慮した保持期間を別途定めてから行う。期限が決まるまでは保持する。
- その他の prefix でもアップロード・検証後に削除候補を列挙する。`rsync --dry-run --delete-unmatched-destination-objects` は候補確認に使い、実際の削除は許可一覧と照合したオブジェクトと generation に限定する。raw data や監査ファイルが候補にあれば停止し、今回の移行では削除しない。現行 `PUBLISHING.md` の混入データも削除する説明は、この方針に合わせて更新する。

### 修復前の内容判定と競合防止

手順3の内容判定は `gcloud storage objects describe` で得る保存オブジェクトの CRC32C とサイズを、ローカルの圧縮ステージから計算した同じ形式の値と比較して行う。generation と metageneration も記録する。修復前のダウンロードは必須とせず、チェックサムで判定してから修復後に保存バイト列を比較する。

| 保存内容 | メタデータ | 処理 |
| --- | --- | --- |
| 一致 | 一致 | 何もしない |
| 一致 | 不一致 | objects update |
| 不一致 | 任意 | 圧縮ステージから cp |
| 判定不能 | 任意 | 修復せず停止 |

`objects update` には取得した `--if-generation-match` と `--if-metageneration-match` を、再アップロードには `--if-generation-match` を指定する。前提条件に失敗したら競合として停止し、最新状態を取得して判定し直す。公開処理は同時実行しない。チェックサム取得・計算の失敗も判定不能として停止する。

修復後は `no-transform` によって GCS 側の自動展開を防ぎ、ダウンロード用の cp・rsync には `--do-not-decompress` を付ける。後者は gcloud 側だけの展開防止であり、GCS 側の自動展開を防ぐ指定ではない。保存バイト列、gzip 展開結果、メタデータを再検証し、失敗したら後続の公開に進まない。

### キャッシュと復旧

移行前に各オブジェクトの Cache-Control を取得し、既定値に依存している場合も実効的な方針を確認して、`no-transform` を追加した値を確定する。復旧用に未圧縮の公開ファイルと元のメタデータを保存し、公開完了の確認まで保持する。

キャッシュ無効のブラウザーで新しい gzip 応答を確認し、通常のキャッシュを使うブラウザーでも表示を確認する。古い未圧縮応答が残る期間は、すべての要求が直ちに gzip になるとは扱わない。同一 URL の複数オブジェクトは一括で切り替わらないため、gzip 移行ではデータ内容を変更せず、内容更新と分ける。

復旧時は、保持した未圧縮の本体と元の Content-Type・Cache-Control をセットで再アップロードし、Content-Encoding が残っていないことを確認する。gzip 本体のまま Content-Encoding だけを削除しない。純資産分布は復旧時もチャンク検証後に manifest を戻す。保存内容・メタデータ・HTTP 応答・画面を再確認し、キャッシュ済み応答も確認する。

## 動作確認と受け入れ条件

- 圧縮前の公開用ファイルと、gzip を展開した圧縮ステージの内容がバイト単位で一致する。
- GCS から取得した保存オブジェクトの gzip 展開結果がローカルの元ファイルと一致する。圧縮されたオブジェクトの検証では、自動展開を無効にして保存バイト列を比較する。
- 同一の圧縮ステージで再実行しても内容とメタデータが正しい状態を保つ。検証用オブジェクトで内容を変えずに `no-transform` などを欠落させた場合も、同期後の修復で期待するメタデータへ戻り、保存バイト列と展開結果の照合が成功する。
- ブラウザーへの HTTP 応答に `Content-Encoding: gzip` があり、Network パネルの転送サイズがリソースサイズより小さい。JSON の解析と全ページのデータ表示が従来どおり動く。
- 代表的な全データ種別に対して初期読み込み・絞り込み・系列比較・詳細表示を確認する。純資産分布では初期 manifest と必要なチャンクを、野菜リレーでは一覧・集計・都道府県詳細を含める。
- 測定記録には圧縮前サイズ、gzip 後の保存サイズ、ブラウザーの転送サイズを区別して記載する。gzip 後のローカルサイズを実転送量の代用にしない。
- 同期の対象 prefix を限定し、公開対象外の raw data や監査ファイルを削除・公開しない。
- 入力が同じ状態で圧縮ステージを再生成して再同期しても、不要な再アップロードが発生しない。
- メタデータだけを修復した検証用オブジェクトは CRC32C と generation が変わらず、metageneration が更新される。修復後の保存バイト列もステージと一致する。
- チャンクの同期・修復・検証中に失敗しても、新しい manifest を公開せず、既存 manifest を削除しない。競合更新、内容判定不能、許可対象外の削除候補では停止する。

## 注意点・除外事項

- gzip は JSON / GeoJSON などテキスト形式に限定する。画像など既に圧縮されたデータには適用しない。
- `Content-Encoding: gzip` だけを付けて実体を未圧縮のままにしたり、`Content-Type: application/gzip` と二重に表現したりしない。圧縮前のデータ型と圧縮符号化をそれぞれ正しく記録する。
- GCS の decompressive transcoding が起きると、応答の `Content-Encoding` / `Content-Length` が変わり、チェックサム照合にも影響する。`no-transform` と保存バイト列の検証を必須にする。
- 圧縮オブジェクトの Range 要求は通常の部分取得として扱えない場合がある。現行アプリは JSON 全体を取得しており Range を使わないことを確認する。
- CORS、キャッシュ設定、ブラウザーまたは中間 CDN の既存キャッシュに古いメタデータの応答が残る可能性を確認する。必要なキャッシュ更新手順は公開運用に合わせて決める。
- ローカル開発サーバーは未圧縮ファイルを返してよい。本番 GCS 応答で gzip 圧縮と JSON の自動展開が機能することを確認する。

## 参考資料

- [Cloud Storage: Transcoding of gzip-compressed files](https://docs.cloud.google.com/storage/docs/transcoding) — `Content-Encoding`、decompressive transcoding、`Cache-Control: no-transform`、Range とチェックサムの注意事項。
- [gcloud storage rsync](https://docs.cloud.google.com/sdk/gcloud/reference/storage/rsync) — 現行同期コマンドの gzip オプションとメタデータ指定。
- [gcloud storage cp](https://docs.cloud.google.com/sdk/gcloud/reference/storage/cp) — manifest の転送、明示的な再アップロード、`--do-not-decompress`。
- [gcloud storage objects update](https://docs.cloud.google.com/sdk/gcloud/reference/storage/objects/update) — 内容を変えずに行うメタデータ修復。
- [Cloud Storage object metadata](https://docs.cloud.google.com/storage/docs/metadata) — `Content-Type` / `Content-Encoding` / `Cache-Control`。
