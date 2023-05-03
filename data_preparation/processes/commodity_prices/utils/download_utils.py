import json
from pathlib import Path
from typing import List

from processes.estat.utils import download_stats_data_as_CSV

# 小売物価統計調査 小売物価統計調査（動向編）
# 主要品目の都市別小売価格－都道府県庁所在市及び人口15万以上の市(2000年1月～)
# https://www.e-stat.go.jp/dbview?sid=0003421913


def get_cat02_codes_from_meta_json(
    meta_json_file_path: Path,
) -> List[str]:
    """指定した json ファイルから cat02_code を取得する関数"""
    with open(meta_json_file_path, "r") as file:
        meta_json = json.load(file)
        class_object_list = meta_json["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"][
            "CLASS_OBJ"
        ]

        # https://stackoverflow.com/questions/9868653/find-first-sequence-item-that-matches-a-criterion
        dict_with_id_of_cat02 = next(
            obj for obj in class_object_list if obj["@id"] == "cat02"
        )
        cat02_dict_list = dict_with_id_of_cat02["CLASS"]
        cat02_code_list = [cat02_dict["@code"] for cat02_dict in cat02_dict_list]

    return cat02_code_list


def download_csv_for_each_cat02_code(
    download_dir: Path,
    estat_app_id: str,
    stats_data_id: str,
    cat02_code_list: List[str],
    section_header: bool,
) -> None:
    """指定した cat02_code の csv をダウンロードする関数"""
    for cat02_code in cat02_code_list:
        file_name = f"{stats_data_id}_{cat02_code}.csv"
        if not Path(download_dir, file_name).exists():
            download_stats_data_as_CSV(
                download_dir,
                file_name,
                estat_app_id,
                stats_data_id,
                section_header,
                cdCat02=cat02_code,
            )
        print(f"Downloaded {file_name}")
