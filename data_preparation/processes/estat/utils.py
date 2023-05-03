import json
from pathlib import Path

import requests


def download_meta_info_as_JSON(
    download_dir: Path,
    file_name: str,
    estat_app_id: str,
    stats_data_id: str,
) -> Path:
    """指定したディレクトリに json を保存し、ファイル名を返す関数"""
    json_file_path = Path(download_dir, file_name)
    if json_file_path.is_file():
        return json_file_path

    result = requests.get(
        f"https://api.e-stat.go.jp/rest/3.0/app/json/getMetaInfo?appId={estat_app_id}&statsDataId={stats_data_id}"
    )
    print(
        f"getMetaInfo -> statsDataId: {stats_data_id}, status code: {result.status_code}"
    )

    with open(
        json_file_path,
        "w",
    ) as file:
        parsed_json = json.loads(result.text)
        json.dump(parsed_json, file, indent=4, ensure_ascii=False)

    return json_file_path


def download_stats_data_as_CSV(
    download_dir: Path,
    file_name: str,
    estat_app_id: str,
    stats_data_id: str,
    section_header: bool,
    **kwargs,
) -> Path:
    """指定したディレクトリに csv を保存し、ファイル名を返す関数"""
    csv_file_path = Path(download_dir, file_name)
    if csv_file_path.is_file():
        return csv_file_path

    if section_header == True:
        sectionHeaderFlg = 1
    else:
        sectionHeaderFlg = 2

    request_url = f"https://api.e-stat.go.jp/rest/3.0/app/getSimpleStatsData?appId={estat_app_id}&statsDataId={stats_data_id}&sectionHeaderFlg={sectionHeaderFlg}"
    for key, value in kwargs.items():
        request_url += f"&{key}={value}"

    result = requests.get(request_url)

    console_message = f"getSimpleStatsData -> statsDataId: {stats_data_id}"
    for key, value in kwargs.items():
        console_message += f", {key}: {value}"

    print(f"{console_message}, status code: {result.status_code}")

    with open(
        csv_file_path,
        "w",
    ) as file:
        file.write(result.text)

    return csv_file_path
