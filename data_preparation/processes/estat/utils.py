import re
from pathlib import Path

import requests


def download_stats_data_as_CSV(
    download_dir: Path,
    file_name: str,
    estat_app_id: str,
    stats_data_id: str,
    section_header: bool,
) -> Path:
    """指定したディレクトリに csv を保存し、ファイル名を返す関数"""
    csv_file_path = Path(download_dir, file_name)
    if csv_file_path.is_file():
        return csv_file_path

    if section_header == True:
        sectionHeaderFlg = 1
    else:
        sectionHeaderFlg = 2

    result = requests.get(
        f"https://api.e-stat.go.jp/rest/3.0/app/getSimpleStatsData?appId={estat_app_id}&statsDataId={stats_data_id}&sectionHeaderFlg={sectionHeaderFlg}"
    )
    print(f"statsDataId: {stats_data_id}, status code: {result.status_code}")

    with open(
        csv_file_path,
        "w",
    ) as file:
        file.write(result.text)

    return csv_file_path
