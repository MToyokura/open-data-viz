# フロントエンド

SvelteKit と Vite を使った Web アプリです。

## 開発環境

Node.js のバージョンは、リポジトリルートの `.node-version` に記載しています。nodenv を使う場合は、リポジトリルートで次を実行してから、`frontend` ディレクトリに移動してください。

```sh
nodenv install
cd frontend
npm ci
npm run dev
```

野菜の産地リレーなどのデータを表示するには、別のターミナルでリポジトリルートから `npm run serve` も実行してください。このサーバーが `data_preparation/output/` のデータを配信します。データの生成方法は [データ準備の README](../data_preparation/README.md) を参照してください。

## ビルドとプレビュー

`frontend` ディレクトリで実行します。

```sh
npm run build
npm run preview
```

## 野菜の産地リレー

ページのパスは `/vegetable_production_relay` です。ページはまず `vegetable_production_relay/products.json` を読み込み、選択された品目の集計データを取得します。地図上で産地を選ぶと、その産地の消費地域別内訳を読み込み、同じページを開いている間はキャッシュします。

表示する数量は、主要消費地域9区分のうち数値が報告された区分の合計です。表示単位未満、該当なし、不詳・調査なしは数値合計に含めず、それぞれの件数を示します。2022年に調査対象が見直されているため、年をまたいだ比較にはご注意ください。データの解釈については[出典と解釈のメモ](../data_preparation/inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md)を参照してください。
