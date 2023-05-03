import csv
import json
import typing
from pathlib import Path

# 数値の単位
# https://www.stat.go.jp/data/kouri/doukou/3.html#meigara

# 説明のページ
# https://www.stat.go.jp/data/kouri/doukou/index.html


def create_time_series_data_for_each_region_from_csv(
    input_csv_file_path: Path,
    output_json_directory: Path,
) -> Path:
    """
    CSV を読み込んで、地域ごとの時系列データを作成する関数。
    CSV が地域ごとに時系列順に並んでいるので iterate して、地域ごとに JSON ファイルを作成する。
    iterate する際、時系列を反転させたいので CSV をファイルの最後から始める。
    """
    with open(input_csv_file_path, "r") as input_csv_file:
        input_csv_file_name = input_csv_file_path.name[0:-4]

        # CSV の先頭行を除外し、逆から読み込む
        reader = reversed(list(csv.reader(input_csv_file))[1:])
        cat02_code = ""
        cat02_name = ""
        previous_area_code = ""
        previous_area_name = ""
        temp_time_series = []

        for row in reader:
            # CSV の最初の数行の例
            # "tab_code","表章項目","cat01_code","データの種別","cat02_code","銘柄","area_code","地域","time_code","時間軸（月）","unit","value","annotation"
            # "10","価格","0020","価格","01001","1001 うるち米(単一原料米,「コシヒカリ」)","01100","札幌市","2023000202","2023年2月","円","2454",""
            # "10","価格","0020","価格","01001","1001 うるち米(単一原料米,「コシヒカリ」)","01100","札幌市","2023000101","2023年1月","円","2454",""

            cat02_code = row[4]
            cat02_name = row[5]
            current_area_code = row[6]
            current_area_name = row[7]

            if current_area_code != previous_area_code:
                # 地域が変わったら、1つ前の地域の時系列データを作成
                if previous_area_code == "":
                    # 1つ目の地域の場合
                    previous_area_code = current_area_code
                    previous_area_name = current_area_name
                    temp_time_series = []
                else:
                    # 2つ目以降の地域の場合

                    # 1つ前の地域の時系列データを作成
                    previous_area_dict = create_area_dict(
                        cat02_code=cat02_code,
                        cat02_name=cat02_name,
                        area_code=previous_area_code,
                        area_name=previous_area_name,
                        time_series=temp_time_series,
                    )

                    # 1つ前の地域の時系列データを JSON ファイルに保存
                    save_output_json_file(
                        output_json_file_path=Path(
                            output_json_directory,
                            f"{input_csv_file_name}_{previous_area_code}.json",
                        ),
                        time_series_data=previous_area_dict,
                    )

                    # 1つ前の地域の情報を更新
                    previous_area_code = current_area_code
                    previous_area_name = current_area_name
                    temp_time_series = []
            time_code = row[8]
            time_name = row[9]
            value = replace_invalid_values_with_none(row[11])
            temp_time_series.append(
                {"time_code": time_code, "time_name": time_name, "value": value}
            )

        # 最後の地域の場合
        previous_area_dict = create_area_dict(
            cat02_code=cat02_code,
            cat02_name=cat02_name,
            area_code=previous_area_code,
            area_name=previous_area_name,
            time_series=temp_time_series,
        )
        save_output_json_file(
            output_json_file_path=Path(
                output_json_directory,
                f"{input_csv_file_name}_{previous_area_code}.json",
            ),
            time_series_data=previous_area_dict,
        )

    return output_json_directory


def save_output_json_file(output_json_file_path: Path, time_series_data: dict) -> None:
    """
    JSON ファイルを保存する関数
    """
    with open(output_json_file_path, "w", encoding="utf8") as output_file:
        json.dump(time_series_data, output_file, ensure_ascii=False)
        print(output_json_file_path)


def create_area_dict(
    cat02_code: str, cat02_name: str, area_code: str, area_name: str, time_series: list
) -> dict:
    """
    地域ごとの時系列データと付属情報をまとめた辞書を作成する関数
    """
    return {
        "cat02_code": cat02_code,
        "cat02_name": cat02_name,
        "area_code": area_code,
        "area_name": area_name,
        "time_series": time_series,
    }


def replace_invalid_values_with_none(original_value: str) -> typing.Union[str, None]:
    # 0003421913_09153_11208.json
    """
    値がない場合は None を返す関数

    以下の文字列が入っている場合は None を返す（estat の凡例表示より）
    ***     数字が得られないもの
    -       調査銘柄の出回りがなかったもの
    ...     動向編：当該市町村で調査を行わないもの、又は調査期間の定めがあるため調査を行わないもの
    """
    if original_value == "***":
        return None
    elif original_value == "-":
        return None
    elif original_value == "...":
        return None
    else:
        return original_value
