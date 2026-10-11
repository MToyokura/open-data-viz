# 野菜リレー調査 API の確認記録

## 現在の解釈（2026-10-11）

表示対象は、[`product_inventory_2021_2023.json`](product_inventory_2021_2023.json) に記録された50品目について、2021～2023年の月別野菜卸売数量とする。150件すべての品目・年別テーブルで、メタデータと観測値の再生検証が完了している。また、公式の期間別ワークブック1,950件すべてを、出典の `cat03` ブロックと照合済みである。品目ごとの対応付けは [`product_temporal_mappings/`](product_temporal_mappings/) にあり、検査結果は [`product_temporal_mapping_report_2021_2023.json`](product_temporal_mapping_report_2021_2023.json) にまとめている。だいこんの元の根拠は [`2021_2023_temporal_evidence.md`](2021_2023_temporal_evidence.md) と [`2021_2023_temporal_mapping.json`](2021_2023_temporal_mapping.json) に記録している。比較を再実行するには `data_preparation/` から `cargo run -- vegetable-relay-verify-daikon-mapping` を実行する。

実務上の解釈と、年次比較に残る論点は [`SOURCE_INTERPRETATION.md`](SOURCE_INTERPRETATION.md) に記録している。要点は、数量の単位はトン（`t`）、`0` は表示単位未満、`－` は事実なし、`…` は不明または未調査である。集計には上位の主要消費地域9地域を含め、その下位地域3地域は含めない。値は報告された産地別の卸売流通量であり、農業生産量の合計ではない。農林水産省によると、2022年に年間取扱量調査の対象が中央卸売市場の卸売会社のみに改定された。この改定が今回の各テーブルに与える影響は確認できていないため、年をまたぐ比較には注意する。

[`product_inventory_2021_2023.json`](product_inventory_2021_2023.json) には、50品目、カタログ上のテーブルID、および2021～2023年のメタデータ・観測値の取得参照を記録している。[`product_workbook_manifest_2021_2023.json`](product_workbook_manifest_2021_2023.json) には、公式ワークブックのURLとチェックサムを記録している。各品目の対応付けは、その品目自身の出典値とワークブックのセルから導出しており、だいこんの根拠を他の品目に流用していない。そらまめでは、2021年と2023年の一部期間に、値が完全一致する出典ブロックが複数ある。候補ブロック間で比較対象セルは同一であり、画面上でもこの対応付けの制約を表示する。

これらは、不完全な状態を明示的に扱うアプリケーション用データである。独立したレビュアーによる承認は必須ではない。以下の過去のM3/M4引き継ぎ記録は、当時の調査状況を示すもの。月との対応が未解決とする記述は、その後の日付付きワークブック根拠と現在の実務方針によって更新されている。

現在の正規化・集計手順は [`M5A_NORMALIZATION.md`](M5A_NORMALIZATION.md) と [`data_preparation/README.md`](../../README.md) のコマンドを参照する。以前の調査・引き継ぎ記録は履歴として以下に残す。

## M4：観測値の完全な取得（JSON形式の出典データ）

`data_preparation/` から、M3のメタデータ取得が**再生検証済み**であることを確認して実行する。

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-acquire inspections/vegetable_production_relay/runs/metadata-<run-id>
cargo run -- vegetable-relay-acquisition-replay inspections/vegetable_production_relay/runs/observations-<run-id> inspections/vegetable_production_relay/runs/metadata-<run-id>
```

ページ分割された `getStatsData` のJSON応答をすべて、SHA-256、ページ位置、返却件数、取得時刻、秘匿情報を除いた台帳メタデータとともに、バイト列そのままで保存する。基本検証をすべてのページが通過した後にのみ台帳を書き込む。オフライン再生では取得データをメタデータのSHA-256に結び付け、欠落・重複・不整合のあるページを拒否し、`-` を含む文字列値、出典の分類コード、単位を保持する。`source_observations.json` は**出典根拠のエクスポート専用**であり、正規化済みデータセットや公開可能なデータセットではない。2021～2023年について月や合計を推定せず、異なる系列間でコードを結合しない。M4計画にある `getSimpleStatsData` のCSVエンコード／スキーマは別途検証が必要であり、この実装ではM3ですでに試したJSON APIを使用する。実行時ファイルはすべてGitの対象外とする。

`data_preparation/` から次を実行する。

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-inspect
cargo run -- vegetable-relay-replay inspections/vegetable_production_relay/runs/<run-id>
```

最初のコマンドは `getStatsList` の応答を1件（最大10テーブル）取得し、`response.bin` にバイト列そのままで保存する。SHA-256チェックサム、秘匿情報を除いたリクエストパラメーター、取得時刻、HTTPステータス、試行回数を `metadata.json` に記録する。統計テーブルの解釈・選定、集計、データ公開は行わない。

2xx以外の応答も保存し、コマンドは失敗終了する。再生検証ではネットワークやアプリケーションIDを使わず、保存したバイト列とチェックサムを検証する。保存された応答のステータスが2xx以外の場合も失敗する。

調査実行にはAPI由来のデータが含まれる場合があり、意図的にGitの対象外としている。フィクスチャをコミットする場合は、内容を確認して秘匿情報を除くこと。`ESTAT_APP_ID` をログや保存ファイルに含めてはならない。一時的なエラーの再試行は最大3回で、中間の失敗応答は保存せず、最後の応答のみを保存する。ネットワークエラーは、認証情報を含む可能性のあるURLを除いて記録する。

## M2-A：カタログの完全な探索

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-discover
cargo run -- vegetable-relay-discovery-replay inspections/vegetable_production_relay/runs/catalogue-<run-id>
```

統計コード `00500226` の `getStatsList` 全ページ（1リクエストあたり100件）を、出典バイト列とページごとのチェックサムを含めて保存する。調査が成功すると、テーブルID、タイトル、基本メタデータを含む検証済み `catalogue.json` を作成する。再生検証では保存した全ページをオフラインで検査し、欠落、重複、チェックサム不一致、総件数の変化、APIエラーを拒否する。統計テーブルの選定は対象外。調査に失敗した場合も、診断用に取得ページをGit対象外の実行ディレクトリへ残すが、検証済みカタログは公開しない。

## M2-B：だいこん候補のオフライン評価

探索とそのオフライン再生検証が成功した後に実行する。

```sh
cargo run -- vegetable-relay-assess inspections/vegetable_production_relay/runs/catalogue-<run-id>
```

このコマンドは生のページ取得データを再生し、保存されたカタログを再生根拠と比較して、Git対象外の実行ディレクトリに `daikon_assessment.json` と `daikon_assessment.md` を作成する。既存の報告書は上書きしない。明示されたタイトルパターンによるだいこん候補の分類と、調査年の手掛かりによるレコードのグループ化のみを行う。タイトルから時間・地理・産地・測定項目を推定せず、テーブルも選定しない。すべての候補は `getMetaInfo` による検証待ちのままとする。秘匿情報を除いた報告書をコミットする場合は、調査結果を確認すること。

## M3：メタデータ目録（意味の承認ではない）

M2の評価を確認した後、候補のテーブルIDを**明示的に**指定する。

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-metadata-inspect <statsDataId>
cargo run -- vegetable-relay-metadata-replay inspections/vegetable_production_relay/runs/metadata-<run-id>
cargo run -- vegetable-relay-metadata-report inspections/vegetable_production_relay/runs/metadata-<run-id>
```

調査では `getMetaInfo` の応答バイト列を `response.bin` と `capture.json` に保存し、出典の分類コード、名称、階層、親参照、存在する場合は出典単位を保持した `inventory.json`／`inventory.md` を作成する。再生検証では応答のチェックサムをオフラインで確認する。報告書は既存ファイルを上書きしない。この目録は**未検証の根拠**として扱い、産地・都道府県・市場・時間の対応付けとみなしてはならない。解釈を文書化し、実際に取得したメタデータと照合して確認するまで、M3のゲートは通過しない。この調査から統計観測値を公開してはならない。

## M3拡張：地域ブロック反復に関する意味的根拠（過去の調査）

2021年（`0002008540`）、2022年（`0002120386`）、2023年（`0002114330`）について、対応する**再生検証済み**のメタデータ実行ディレクトリを使う。

```sh
cargo run -- vegetable-relay-semantic-blocks inspections/vegetable_production_relay/runs/metadata-<run-id>
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-semantic-probe inspections/vegetable_production_relay/runs/metadata-<run-id> 1001
cargo run -- vegetable-relay-semantic-replay inspections/vegetable_production_relay/runs/semantic-probe-<run-id> inspections/vegetable_production_relay/runs/metadata-<run-id>
```

`semantic_blocks.md` は、同じ12地域ラベルが13回反復するブロックについて、月名を割り当てずに出典コードの正確な範囲を記録する。プローブは有効な `cat03` コードを1つ明示的に絞り込み、`getStatsData` の行を最大10件要求する。最後の応答バイト列、チェックサム、秘匿情報を除いたリクエストメタデータを保存し、同一のメタデータ取得データに対してオフライン検証できる。`sample.json` は生の観測参照と文字列値を保持するだけであり、**正規化済みデータセットではない**。失敗時も根拠はGit対象外の実行ディレクトリに残る。出典値は機微情報を含み得るため、コミット前に秘匿情報を除くこと。ブロックと月の対応、合計ブロックの識別、地理的範囲、単位の比較可能性は未検証のままであり、限定的なサンプルだけではそれらを証明できない。候補選定やM4でのデータ公開は行わない。

## M3からM4への引き継ぎ判断：時間的な意味が未確定のままM4へ進む（過去の記録）

**判断（2026-10-11）：** M3の調査ツールと代表的な根拠収集が完了したため、M4の取得作業を開始できる。これは**範囲を限定した技術上の引き継ぎ**であり、すべての統計的解釈を承認するものではない。`Gate: BLOCKED` とする過去のM3コマンド出力や報告は有効であり、曖昧な2021～2023年のブロックに関する**意味の承認は依然として保留**である。このメッセージを取得／再生検証の失敗や、データ品質の合格と解釈してはならない。

収集した根拠：

- **2021～2023年：** 統合候補テーブル `0002008540`、`0002120386`、`0002114330` はそれぞれ156件の `cat03` コードを持つ。明示的な `time` 軸はなく、同じ12地域ラベルと階層レベルが13回連続している。2023年の `TABLE_INF.SURVEY_DATE` は `202301-202312`、タイトルは `計_1月～12月` である。これらは全体の対象期間を示すが、各ブロックの月や合計の識別根拠には**ならない**。
- **2023年のAPI確認：** `getMetaInfo` と件数を制限した `getStatsData` のプローブは成功した。観測値の取得データは、対応するメタデータ取得データと照合して再生検証に成功した。プローブした観測値には分類参照、単位、文字列値（`-` を含む）があったが、それとは独立した月対応情報はなかった。`explanationGetFlg=Y` を指定した `getMetaInfo` の要求も行ったが、確認した項目ではテーブル説明は空で、分類オブジェクトの説明や追加属性もなかった。e-Statの画面とエクスポートしたワークブックからも、13ブロックを月別に識別できなかった。

### M4での要件：意味が未確定なら処理を止める

1. M4では、これらのテーブルについて、出典テーブルID、分類コード、文字列値（数値以外の記号を含む）、応答チェックサム、来歴を保った生の観測値を取得して再生検証してよい。取得は月別の正規化や公開を意味しない。
2. 2021～2023年の13ブロックは、出典の `cat03` コードまたはブロックの序数だけで表す。`reporting_month` と年間合計との対応は**不明**のままとし、ブロック順、ラベルの反復、`SURVEY_DATE`、タイトル、算術的なパターン（`156 = 13 × 12`）から導出してはならない。
3. 類似したテーブル形状を根拠に、対象年を追加で推定してはならない。対象は2021～2023年のみ。
4. 出典系列をまたいで、生の識別子だけを根拠にコードを結合してはならない（例：2021年より前の `001` とそれ以降の `1001`）。類似ラベルのみから、地理、包含／除外規則、単位の同等性、測定値の比較可能性を推定してはならない。出典の単位と欠損・秘匿記号を保持し、`-` を**ゼロとして扱わない**。
5. 独立して引用可能な出典根拠が `cat03` ブロックから月（および合計）への完全な対応付けを示し、レビュアーが承認するまで、2021～2023年データの月別正規化と公開を下流工程で**停止**する。出典参照、対応付けの根拠、検証結果を判断とともに記録する。未検証の対応付けを既定値として暗黙に使ってはならない。

**状況：** 代表的なテーブルのM3取得／再生検証と構造調査：**完了**。M3の年をまたぐ意味の検証：**一部未解決**。M4の生データ取得：**これらの制約の下で実施可能**。2021～2023年の観測値の月別公開：**保留**。
