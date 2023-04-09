データを取得・加工するディレクトリです。

## 使い方

以下の手順で、サイトで使用しているすべてのデータを取得できます。

1. [Poetry](https://python-poetry.org/docs/#installation) をインストールして `poetry install` を実行する
1. `./processes` で `poetry run python -m luigi --module luigi_module AllTasks --local-scheduler` を実行する

各データの処理はディレクトリごとに別れています。各ディレクトリにはデータを入れておくフォルダと、実行される処理が入っています。

データを入れておくフォルダは 3 つあります。

```
raw_data           # 生のデータ
intermediate_data  # 途中で生じたデータ
final_data         # フロントで使用するデータ
```
