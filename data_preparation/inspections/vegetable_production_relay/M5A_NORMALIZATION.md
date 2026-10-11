# だいこんの正規化（2021～2023年）

取得済みのe-Stat行データ、対応するメタデータ、公式ワークブックによる対応付けから、正規化レコードを生成する。現在の出典の意味と単位は [`SOURCE_INTERPRETATION.md`](SOURCE_INTERPRETATION.md) を参照。

## オフライン検査

`data_preparation/` から次を実行する。

```sh
cargo run -- vegetable-relay-validate-m3-m5
```

取得データを再生検証し、日付付きワークブックとの対応付けを確認して、3テーブルすべてを正規化する。出典上の位置、コード、値、期間を検査する。入力または対応付けの検査に失敗した場合は、失敗終了する。独立したレビュアー記録は必要としない。

## 正規化済み行の生成

各年について、取得した観測値と対応するメタデータ、共通の時間対応付けを指定し、新しい出力ディレクトリを使って正規化する。

```sh
cargo run -- vegetable-relay-normalize-2021-2023 \
  <observations-run-dir> <metadata-run-dir> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  <new-output-dir>
```

出力には、出典テーブル、`cat03` コード、期間、生の文字列値、出典メタデータの単位を保持する。2021年のAPI行では単位情報はないままとし、集計ではワークブックで確認した表示単位 `t` を使う。

`0`、`-`、`…` はそれぞれ `below_display_unit`、`no_fact`、`unknown_or_uncollected` に正規化する。年間の `計` ブロックは根拠として保持し、月別集計から除外する。集計では、階層レベル1の主要消費地域9地域について、数量の数値だけを合計する。不完全なセルは別々に件数を数える。
