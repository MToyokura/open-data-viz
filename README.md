# Open Data Viz

[Open Data Viz](https://mtoyokura.github.io/open-data-viz) のソースコードです。統計データの取得・加工と、SvelteKit による可視化を行います。

## プロジェクト構成

- `frontend/`: SvelteKit / Svelte 5 のウェブアプリ
- `data_preparation/`: e-Stat の統計データと国土数値情報を取得・加工する Rust プログラム
- `data_preparation/inspections/`: 野菜データの出典確認記録と品目別の時間対応付け

## 開発

ローカルでの起動、ページの説明、ビルド方法は[フロントエンドのREADME](frontend/README.md)、データ生成は[データ準備のREADME](data_preparation/README.md)を参照してください。

本サイトは [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/deed.ja) で公開しています。
