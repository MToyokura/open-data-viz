import os
import subprocess
from pathlib import Path

import luigi
from constants.path_constants import workers_by_age_employment_type_education_sex_path
from processes.luigi_utils import initialize_data_folder

raw_data_dir = Path(workers_by_age_employment_type_education_sex_path, "raw_data")
intermediate_data_dir = Path(
    workers_by_age_employment_type_education_sex_path, "intermediate_data"
)
final_data_dir = Path(workers_by_age_employment_type_education_sex_path, "final_data")


class InitializeDataFolder(luigi.Task):
    def run(self):
        initialize_data_folder(workers_by_age_employment_type_education_sex_path)

    def complete(self):
        if (
            Path(raw_data_dir).exists()
            and Path(intermediate_data_dir).exists()
            and Path(final_data_dir).exists()
        ):
            return True
        return False


class DownloadEmploymentDataFromEstat(luigi.Task):
    def requires(self):
        return InitializeDataFolder()

    def run(self):
        subprocess.run(
            [
                "poetry",
                "run",
                "python",
                f"{Path(workers_by_age_employment_type_education_sex_path, 'download_from_estat_and_save_individual_json.py')}",
            ]
        )

    def output(self):
        # 簡易的に最後に作成されるファイルを出力とする
        return luigi.LocalTarget(Path(final_data_dir, "20121002.json"))


class ExportFinalDataToFrontend(luigi.Task):
    # TODO: constants のパスを使う

    frontend_data_dir = Path(
        "../../frontend/public/assets/workers_by_age_employment_type_education_sex_files/"
    )

    def requires(self):
        return DownloadEmploymentDataFromEstat()

    def run(self):
        if not self.frontend_data_dir.exists():
            self.frontend_data_dir.mkdir(parents=True)

        subprocess.run(["cp", "-r", f"{str(final_data_dir)}/.", self.frontend_data_dir])

    def output(self):
        return luigi.LocalTarget(Path(self.frontend_data_dir, "20121002.json"))


class ListFiles(luigi.Task):
    files_list_path = Path(
        workers_by_age_employment_type_education_sex_path, "files_list.txt"
    )
    files_list_for_comparison_path = Path(
        workers_by_age_employment_type_education_sex_path,
        "files_list_for_comparison.txt",
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
                [
                    "ls",
                    "-R",
                    "processes/workers_by_age_employment_type_education_sex/raw_data",
                ],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            intermediate_data = subprocess.run(
                [
                    "ls",
                    "-R",
                    "processes/workers_by_age_employment_type_education_sex/intermediate_data",
                ],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            final_data = subprocess.run(
                [
                    "ls",
                    "-R",
                    "processes/workers_by_age_employment_type_education_sex/final_data",
                ],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            output = f"{raw_data}\n{intermediate_data}\n{final_data}"
            output_file.write(output)
            # レポジトリをクローンしたときにデータが揃っているか確認するために & luigi を実行するために
            # gitignore される、中身が同一のファイルを作成する
            output_file_for_comparison.write(output)

    def output(self):
        return luigi.LocalTarget(self.files_list_for_comparison_path)
