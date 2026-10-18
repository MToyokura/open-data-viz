# 野菜リレーの実験・調査記録

このフォルダでは、野菜の産地リレーの可視化に向けて、どの出典データを使い、年月や数量をどう解釈し、どのように加工・集計するかを実験・調査しています。候補データの照合結果や加工方法の検討記録と、そこから採用した品目台帳・年月の対応表を保存しています。

現在の生成処理は、このフォルダの品目台帳と年月の対応表を参照しています。以下では、実験・調査で得られた結果と、生成処理に使われているファイルの関係を説明します。

## 現在の状態

対象は [`product_inventory_2021_2023.json`](product_inventory_2021_2023.json) に記録した50品目の2021～2023年データです。150件の品目・年別テーブルについて e-Stat metadata と観測値を再生検証し、公式の期間別ワークブック1,950件を各テーブルの `cat03` ブロックと照合しました。各品目の対応付けは [`product_temporal_mappings/`](product_temporal_mappings/) に、検査結果の一覧は [`product_temporal_mapping_report_2021_2023.json`](product_temporal_mapping_report_2021_2023.json) にあります。

ワークブックのURLとチェックサムは [`product_workbook_manifest_2021_2023.json`](product_workbook_manifest_2021_2023.json) に記録しています。出典から再検証する手順は、以下の[mapping の再検査](#mapping-の再検査)を参照してください。

## 調査結果の読み方

- 月対応は品目ごとの公式ワークブックと出典値から導出しており、だいこんの対応を他品目に流用していません。
- そらまめの一部期間では、比較値が完全一致する複数の候補ブロックがあります。候補間で表示値は同じですが、対応付けの曖昧さはmappingに記録し、画面にも示します。

数量の単位・記号・集計範囲と年次比較上の注意は [`SOURCE_INTERPRETATION.md`](SOURCE_INTERPRETATION.md)、正規化・集計仕様は [`M5A_NORMALIZATION.md`](M5A_NORMALIZATION.md) を参照してください。だいこんの時間対応付けの具体的な根拠は [`2021_2023_temporal_evidence.md`](2021_2023_temporal_evidence.md) にあります。

## ファイルと生成処理

### inventory と temporal mapping

[`product_inventory_2021_2023.json`](product_inventory_2021_2023.json) は品目台帳です。品目キー・表示名・対象年ごとの e-Stat テーブルIDなどを記録しています。ファイル名にある `inventory` は「棚卸し・一覧」の意味で、観測値の本体ではありません。

各品目の [`product_temporal_mappings/`](product_temporal_mappings/) にあるJSONは、e-Statの `cat03` コードブロックと「計」または各月との対応表です。e-Statの表に月の軸がないため、公式ワークブックの見出しとセル値を照合して対応を確認しています。inventory と mapping はGit管理されており、clone後も利用できます。

### capture と生成データ

capture は e-Stat API から取得したmetadataまたは観測値と、その出典・チェックサムなどの検証情報を保存したものです。生成コマンドは既存の有効なcaptureを再利用し、見つからない場合はAPIから取得します。captureと公式ワークブックの取得ファイルは `runs/` 以下に保存されます。この実行時データと、生成後の `data_preparation/output/` はGit管理対象外です。そのため、clone直後は生成済みデータを配信できません。実行コマンドと必要条件は[データ準備のREADME](../../README.md#野菜の産地別卸売データ)を参照してください。

この処理は50品目の観測値を取得または再利用し、検証・正規化・集計して `output/vegetable_production_relay/` に書き出します。配信用のsummary、detail、audit、互換JSON、品目索引は毎回作り直し、既存の配信用ファイルは読み込まず再利用しません。生成開始時にこの処理が管理する品目ファイルと品目ディレクトリだけを削除します。同じ出力ディレクトリの他のファイルは残します。生成に失敗した場合は出力が不完全な状態になり得るため、修正後に生成コマンドを再実行してください。主なファイルは次のとおりです。

- `products.json`: フロントエンドが読む品目一覧
- `{品目キー}.summary.json`: 地図用の月別・都道府県別集計（schema version 2）
- `{品目キー}/details/{都道府県コード}.json`: 都道府県選択後に読む主要消費地域9区分の内訳（schema version 2）
- `{品目キー}/audit.json`: 検証ハッシュと出典セル位置
- `{品目キー}.json`: 旧schema version 1形式の互換ファイル

`products.json` の更新時にsummary、47都道府県分のdetail、auditの整合性を検査します。不完全な品目は一覧に含めません。

### mapping の再検査

通常の生成では、Git管理されているmappingを使うためワークブックの再取得は不要です。出典からmappingを再検証する場合は、`data_preparation/` で次の手順を実行します。

```sh
cargo run -- vegetable-relay-inspect-workbooks
export ESTAT_APP_ID="取得したアプリケーションID"
cargo run -- vegetable-relay-acquire-product-observations
cargo run -- vegetable-relay-verify-product-mappings
```

最初のコマンドは公式ワークブックを取得するためネット接続が必要です。2つ目はmetadataと観測値を取得または再利用し、inventoryのcapture参照を更新します。このコマンドには `ESTAT_APP_ID` が必要で、未取得のデータがある場合はネット接続も必要です。最後のコマンドはローカルのワークブックと観測値を照合してmappingを検証します。
