import os
import subprocess
from pathlib import Path

import luigi
from processes.luigi_utils import initialize_data_folder

current_file_location = Path(os.path.dirname(os.path.realpath(__file__)))
raw_data_dir = Path(current_file_location, "raw_data")
intermediate_data_dir = Path(current_file_location, "intermediate_data")
final_data_dir = Path(current_file_location, "final_data")


class InitializeDataFolder(luigi.Task):
    def run(self):
        initialize_data_folder(current_file_location)

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
                f"{Path(current_file_location, 'download_from_estat_and_save_individual_json.py')}",
            ]
        )

    def output(self):
        # 簡易的に最後に作成されるファイルを出力とする
        return luigi.LocalTarget(Path(final_data_dir, "20121002.json"))


class ExportFinalDataToFrontend(luigi.Task):
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
    files_list_path = Path(current_file_location, "files_list.txt")
    files_list_for_comparison_path = Path(
        current_file_location, "files_list_for_comparison.txt"
    )

    def requires(self):
        return ExportFinalDataToFrontend()

    def run(self):
        with open(file=self.files_list_path, mode="w") as output_file, open(
            file=self.files_list_for_comparison_path, mode="w"
        ) as output_file_for_comparison:
            raw_data = subprocess.run(
                ["ls", "-R", f"{current_file_location}/raw_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            intermediate_data = subprocess.run(
                ["ls", "-R", f"{current_file_location}/intermediate_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            final_data = subprocess.run(
                ["ls", "-R", f"{current_file_location}/final_data"],
                stdout=subprocess.PIPE,
            ).stdout.decode()
            output = f"{raw_data}\n{intermediate_data}\n{final_data}"
            output_file.write(output)
            # レポジトリをクローンしたときにデータが揃っているか確認するために & luigi を実行するために
            # gitignore される、中身が同一のファイルを作成する
            output_file_for_comparison.write(output)

    def output(self):
        return luigi.LocalTarget(self.files_list_for_comparison_path)
