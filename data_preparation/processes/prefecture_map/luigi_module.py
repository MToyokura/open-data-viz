# 元データ
# https://data.humdata.org/dataset/cod-ab-jpn
# https://data.humdata.org/dataset/6ba099c6-350b-4711-9a65-d85a1c5e519c/resource/f82faadf-a608-42cf-ae15-75ce672d7e69/download/jpn_adm_2019_shp.zip
# Contributor: OCHA Regional Office for Asia and the Pacific (ROAP)
# The original shapefile is licensed under a Creative Commons Attribution for Intergovernmental Organisations license.

# 参考文献
# https://qiita.com/cieloazul310/items/9d0e3f67f60d791726dc
# https://curl.se/docs/manpage.html#-L
# https://stackoverflow.com/questions/9679932/how-to-use-executables-from-a-package-installed-locally-in-node-modules
# https://github.com/mbostock/shapefile/blob/master/README.md#shp2json
# https://github.com/topojson/topojson-server
# https://github.com/topojson/topojson-simplify
# https://github.com/topojson/topojson-client
# https://bost.ocks.org/mike/make/
# https://makefiletutorial.com/


import os
import shutil
import subprocess
import typing
from pathlib import Path

import luigi
import requests
from processes.luigi_utils import initialize_data_folder
from processes.prefecture_map.omit_islands import omit_islands

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


class DownloadShapeFileZip(luigi.Task):
    output_file_path = Path(raw_data_dir, "jpn_adm_2019_shp.zip")

    def requires(self):
        return InitializeDataFolder()

    def run(self):
        url = "https://data.humdata.org/dataset/6ba099c6-350b-4711-9a65-d85a1c5e519c/resource/f82faadf-a608-42cf-ae15-75ce672d7e69/download/jpn_adm_2019_shp.zip"
        response = requests.get(url)
        with open(file=self.output_file_path, mode="wb") as f:
            f.write(response.content)

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class UnzipShapeFileZip(luigi.Task):
    output_file_path = Path(raw_data_dir, "jpn_admbnda_adm1_2019.shp")

    def requires(self):
        return DownloadShapeFileZip()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        shutil.unpack_archive(filename=luigi_input.path, extract_dir=Path(raw_data_dir))

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class ConvertShapefileToGeoJson(luigi.Task):
    output_file_path = Path(intermediate_data_dir, "prefectures.geojson")

    def requires(self):
        return UnzipShapeFileZip()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        with open(file=self.output_file_path, mode="w") as output_file:
            subprocess.run(
                # yarn だとファイルパス指定で変な動きをするので npx を使う
                # https://github.com/mbostock/shapefile#shp2json
                [
                    "npx",
                    "shp2json",
                    "--encoding",
                    "UTF8",
                    luigi_input.path,
                ],
                stdout=output_file,
            )

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class ConvertGeoJsonToTopoJson(luigi.Task):
    output_file_path = Path(intermediate_data_dir, "prefectures_topo.json")

    def requires(self):
        return ConvertShapefileToGeoJson()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        with open(file=self.output_file_path, mode="w") as output_file:
            subprocess.run(
                # yarn だとファイルパス指定で変な動きをするので npx を使う
                # https://github.com/topojson/topojson-server/blob/master/README.md#geo2topo
                ["npx", "geo2topo", "-q", "1e6", f"japan={luigi_input.path}"],
                stdout=output_file,
            )

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class SimplifyTopoJson(luigi.Task):
    output_file_path = Path(intermediate_data_dir, "prefectures_topo_simplified.json")

    def requires(self):
        return ConvertGeoJsonToTopoJson()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        with open(
            file=self.output_file_path,
            mode="w",
        ) as output_file:
            subprocess.run(
                # yarn だとファイルパス指定で変な動きをするので npx を使う
                # https://github.com/topojson/topojson-simplify#toposimplify
                ["npx", "toposimplify", "-P", "0.05", "-f", luigi_input.path],
                stdout=output_file,
            )

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class ConvertSimplifiedTopoJsonToGeoJson(luigi.Task):
    output_file_path = Path(intermediate_data_dir, "prefectures_simplified.geojson")

    def requires(self):
        return SimplifyTopoJson()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        with open(file=luigi_input.path, mode="r") as input_file:
            subprocess.run(
                # yarn だとファイルパス指定で変な動きをするので npx を使う
                # https://github.com/topojson/topojson-client#topo2geo
                [
                    "npx",
                    "topo2geo",
                    f"japan={self.output_file_path}",
                ],
                stdin=input_file,
            )

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class OmitIslands(luigi.Task):
    output_file_path = Path(final_data_dir, "simplified_geojson_island_omitted.geojson")

    def requires(self):
        return ConvertSimplifiedTopoJsonToGeoJson()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        omit_islands(
            input_file_path=luigi_input.path, output_file_path=self.output_file_path
        )

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


class ExportFinalDataToFrontend(luigi.Task):
    output_file_path = Path(
        "../../frontend/public/assets/simplified_geojson_island_omitted.geojson"
    )

    def requires(self):
        return OmitIslands()

    def run(self):
        luigi_input: typing.Any = self.input()  # 本当は type は luigi.LocalTarget
        subprocess.run(["cp", luigi_input.path, self.output_file_path])

    def output(self):
        return luigi.LocalTarget(self.output_file_path)


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
                ["ls", "-R", "prefecture_map/raw_data"], stdout=subprocess.PIPE
            ).stdout.decode()
            intermediate_data = subprocess.run(
                ["ls", "-R", "prefecture_map/intermediate_data"], stdout=subprocess.PIPE
            ).stdout.decode()
            final_data = subprocess.run(
                ["ls", "-R", "prefecture_map/final_data"], stdout=subprocess.PIPE
            ).stdout.decode()
            output = f"{raw_data}\n{intermediate_data}\n{final_data}"
            output_file.write(output)
            # レポジトリをクローンしたときにデータが揃っているか確認するために & luigi を実行するために
            # gitignore される、中身が同一のファイルを作成する
            output_file_for_comparison.write(output)

    def output(self):
        return luigi.LocalTarget(self.files_list_for_comparison_path)
