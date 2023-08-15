import os
import subprocess
from pathlib import Path
from typing import List

import luigi
from constants.path_constants import commodity_prices_path
from processes.commodity_prices.utils.create_autoselect_data import (
    create_autoselect_data,
)
from processes.commodity_prices.utils.download_utils import (
    download_csv_for_each_cat02_code,
    get_cat02_codes_from_meta_json,
)
from processes.commodity_prices.utils.process_utils import (
    create_time_series_data_for_each_region_from_csv,
)
from processes.estat.utils import download_meta_info_as_JSON
from processes.estat.variables import ESTAT_APP_ID
from processes.luigi_utils import initialize_data_folder

raw_data_dir = Path(commodity_prices_path, "raw_data")
intermediate_data_dir = Path(commodity_prices_path, "intermediate_data")
final_data_dir = Path(commodity_prices_path, "final_data")
final_autoselect_data_dir = Path(final_data_dir, "autoselect_data")
final_prices_data_dir = Path(final_data_dir, "commodity_price_json_files")


class InitializeDataFolder(luigi.Task):
    def run(self):
        initialize_data_folder(commodity_prices_path)

    def output(self):
        return luigi.LocalTarget(raw_data_dir)


class DownloadMetaInfoJsonFromEstat(luigi.Task):
    def requires(self):
        return InitializeDataFolder()

    def run(self):
        download_meta_info_as_JSON(
            download_dir=raw_data_dir,
            file_name="0003421913_meta.json",
            estat_app_id=ESTAT_APP_ID,
            stats_data_id="0003421913",
        )

    def output(self):
        return luigi.LocalTarget(Path(raw_data_dir, "0003421913_meta.json"))


class DownloadCsvFromEstat(luigi.Task):
    def requires(self):
        return DownloadMetaInfoJsonFromEstat()

    def run(self):
        cat02_codes: List[str] = get_cat02_codes_from_meta_json(
            Path(raw_data_dir, "0003421913_meta.json")
        )
        if not Path(raw_data_dir, "csv").exists():
            Path(raw_data_dir, "csv").mkdir()
        download_csv_for_each_cat02_code(
            download_dir=Path(raw_data_dir, "csv"),
            estat_app_id=ESTAT_APP_ID,
            stats_data_id="0003421913",
            cat02_code_list=cat02_codes,
            section_header=False,
        )

    def output(self):
        # 簡易的に最後にダウンロードされるファイルを出力とする
        return luigi.LocalTarget(Path(raw_data_dir, "csv", "0003421913_09962.csv"))


class CreateAndSaveFinalData(luigi.Task):
    def requires(self):
        return DownloadCsvFromEstat()

    def run(self):
        if not final_prices_data_dir.exists():
            final_prices_data_dir.mkdir(parents=True)
        # raw_data_dir/csv 以下の全ての csv ファイルを読み込み、
        # final_data_dir 以下に json ファイルを作成する
        for csv_file in Path(raw_data_dir, "csv").iterdir():
            create_time_series_data_for_each_region_from_csv(
                input_csv_file_path=csv_file,
                output_json_directory=final_prices_data_dir,
            )

    def output(self):
        # 簡易的に最後にダウンロードされるファイルを出力とする
        return luigi.LocalTarget(
            Path(final_prices_data_dir, "0003421913_09962_47201.json")
        )


class CreateAutoselectCommodityData(luigi.Task):
    def requires(self):
        return CreateAndSaveFinalData()

    def run(self):
        if not final_autoselect_data_dir.exists():
            final_autoselect_data_dir.mkdir()
        create_autoselect_data(
            Path(raw_data_dir, "0003421913_meta.json"), Path(final_autoselect_data_dir)
        )

    def output(self):
        return luigi.LocalTarget(
            Path(final_autoselect_data_dir, "commodity_selections.json")
        )


class CreateAutoselectRegionData(luigi.Task):
    def requires(self):
        return CreateAutoselectCommodityData()

    def output(self):
        return luigi.LocalTarget(
            Path(final_autoselect_data_dir, "region_selections.json")
        )


class ExportFinalDataToFrontend(luigi.Task):
    # TODO: constants のパスを使う

    frontend_data_dir = Path("../../frontend/public/assets/commodity_prices/")

    def requires(self):
        return CreateAutoselectRegionData()

    def run(self):
        if not self.frontend_data_dir.exists():
            self.frontend_data_dir.mkdir(parents=True)

        subprocess.run(["cp", "-r", f"{str(final_data_dir)}/.", self.frontend_data_dir])

    def output(self):
        return luigi.LocalTarget(
            # 簡易的に最後にダウンロードされるファイルを出力とする
            Path(
                self.frontend_data_dir,
                "commodity_price_json_files",
                "0003421913_09962_47201.json",
            )
        )


class ListFiles(luigi.Task):
    files_list_path = Path(commodity_prices_path, "files_list.txt")
    files_list_for_comparison_path = Path(
        commodity_prices_path, "files_list_for_comparison.txt"
    )

    def requires(self):
        return ExportFinalDataToFrontend()

    def run(self):
        with open(file=self.files_list_path, mode="w") as output_file, open(
            file=self.files_list_for_comparison_path, mode="w"
        ) as output_file_for_comparison:
            # システムの絶対パスを使うとユーザー名とかが表示されてしまうので、
            # luigi が実行されるディレクトリからのパスを手書きする。
            raw_data = subprocess.run(
                ["ls", "-R", "processes/commodity_prices/raw_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            intermediate_data = subprocess.run(
                ["ls", "-R", "processes/commodity_prices/intermediate_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            final_data = subprocess.run(
                ["ls", "-R", "processes/commodity_prices/final_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            output = f"{raw_data}\n{intermediate_data}\n{final_data}"
            output_file.write(output)
            # レポジトリをクローンしたときにデータが揃っているか確認するために & luigi を実行するために
            # gitignore される、中身が同一のファイルを作成する
            output_file_for_comparison.write(output)

    def output(self):
        return luigi.LocalTarget(self.files_list_for_comparison_path)
