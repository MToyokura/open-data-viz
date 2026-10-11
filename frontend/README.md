# フロントエンド

SvelteKit、Svelte 5、Vite を使ったウェブアプリです。

## 開発環境

Node.js のバージョンはリポジトリルートの `.node-version` に記載しています。`frontend/` で依存パッケージをインストールして開発サーバーを起動します。

```sh
cd frontend
npm ci
npm run dev
```

統計データと地図データを表示するには、リポジトリルートでもう一つターミナルを開き、データ配信用サーバーを起動してください。

```sh
# リポジトリルート
npm ci
npm run serve
```

サーバーは `data_preparation/output/` を `http://localhost:8000` で配信します。野菜データや都道府県 GeoJSON が未生成の場合は、[データ準備のREADME](../data_preparation/README.md)を参照してください。

データ配信先のベースURLは `VITE_STORAGE_BASE_URL` で変更できます。既定値は `http://127.0.0.1:8000` です。パス付きの本番配置では、SvelteKit のルートに `BASE_PATH`、トップページ内リンクに `VITE_ROOT_PATH` を設定します。GitHub Pages 用の設定例は [deploy-sveltekit.yml](../.github/workflows/deploy-sveltekit.yml) にあります。

## ビルドとプレビュー

`frontend/` で実行します。

```sh
npm run build
npm run preview
```

## ページ

- `/commodity_prices`: 品目と都市を選び、小売価格の系列をグラフに追加して比較します。
- `/vegetable_production_relay`: 品目と年月を選び、都道府県ごとの報告数量を地図で見ます。地図上の都道府県を選ぶと主要消費地域9区分の内訳を読み込みます。同じページを開いている間、取得済みの内訳はキャッシュされます。

産地リレーの集計方法と出典の解釈は[野菜リレー調査記録](../data_preparation/inspections/vegetable_production_relay/README.md#調査結果の読み方)を参照してください。
