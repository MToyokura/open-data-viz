import json
from pathlib import Path

import pandas


def create_json_files(csv_df: pandas.DataFrame, processed_data_dir: Path):
    for cat01_code in pandas.unique(csv_df["cat01_code"]):
        for cat03_code in pandas.unique(csv_df["cat03_code"]):
            for cat04_code in pandas.unique(csv_df["cat04_code"]):
                for cat05_code in pandas.unique(csv_df["cat05_code"]):
                    code_id = (
                        str(cat01_code).zfill(2)
                        + str(cat03_code).zfill(2)
                        + str(cat04_code).zfill(2)
                        + str(cat05_code).zfill(2)
                    )

                    output_file_path = Path(processed_data_dir, f"{code_id}.json")
                    if output_file_path.is_file():
                        print(output_file_path)
                        continue

                    # df を条件のものに絞り込む
                    current_df = csv_df[
                        (csv_df["cat01_code"] == cat01_code)
                        & (csv_df["cat03_code"] == cat03_code)
                        & (csv_df["cat04_code"] == cat04_code)
                        & (csv_df["cat05_code"] == cat05_code)
                    ]

                    if current_df.empty:
                        continue

                    data_dict = {
                        "id": code_id,
                        "年齢階級(詳細集計）": pandas.unique(current_df["年齢階級(詳細集計）"])[0],
                        "雇用形態": pandas.unique(current_df["雇用形態"])[0],
                        "教育": create_education_label(str(cat04_code).zfill(2)),
                        "性別": pandas.unique(current_df["性別"])[0],
                        "data": dict(zip(current_df["時間軸（年次）"], current_df["value"])),
                    }

                    with open(output_file_path, "w") as file:
                        file.write(json.dumps(data_dict))
                        print(output_file_path)


def create_education_label(cat04_code: str) -> str:
    # 「教育」の在学中／卒業の表記がひどいので自力でつける
    match cat04_code:
        case "00":
            return "総数"
        case "01":
            return "在学中計"
        case "02":
            return "小学・中学・高校（在学中）"
        case "03":
            return "短大・高専（在学中）"
        case "04":
            return "大学・大学院（在学中）"
        case "05":
            return "卒業計"
        case "06":
            return "小学・中学・高校・旧中（卒業）"
        case "07":
            return "短大・高専（卒業）"
        case "08":
            return "大学・大学院（卒業）"
        case "09":
            return "大学（卒業）"
        case "10":
            return "大学院（卒業）"
        case _:
            raise ValueError(f"{cat04_code} does not exist in cat04_code")
