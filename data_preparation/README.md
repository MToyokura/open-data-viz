データを取得・加工するディレクトリです。

## 使い方

### データの取得

以下の手順で、サイトで使用しているすべてのデータを取得できます。

1. [Poetry](https://python-poetry.org/docs/#installation) をインストールして `poetry install` を実行する
1. [Yarn](https://chore-update--yarnpkg.netlify.app/ja/docs/install) をインストールして `yarn install` を実行する
1. `ESTAT_APP_ID` という環境変数に自身の[ e-Stat のアプリケーション ID](https://www.e-stat.go.jp/api/api-dev/faq#q_3)を設定する
1. `poetry run python -m luigi --module luigi_module AllTasks --local-scheduler` を実行する

各データの取得および加工の処理はディレクトリごとに別れています。各ディレクトリにはデータを入れておくフォルダと、実行される処理が入っています。

データを入れておくフォルダは 3 つあります。

```
raw_data           # 生のデータ
intermediate_data  # 途中で生じたデータ
final_data         # フロントで使用するデータ
```

### テストの実行

```bash
poetry run python -m unittest
```

## `files_list.txt` について

`processes/` の各データ取得系フォルダ内に `files_list.txt` というファイルがあります。これらにはデータ取得後に各フォルダ内に存在するべきファイルの一覧が記載されています。

ローカルでデータの取得を実行すると `files_list_for_comparison.txt` というファイルも同時に作成されます。内容を `files_list.txt` と比較することで、 GitHub 上で公開されている時点のデータとの差分を取ることができます。
