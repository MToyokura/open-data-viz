import copy
import json
from pathlib import Path

import pandas
import processes.estat.utils as estat_utils
import processes.monthly_vegetable_market_amount_by_prefecture.my_types as my_types
import requests
from processes.estat.variables import ESTAT_APP_ID


def get_and_save_json_response(
    estat_app_id: str, download_dir: Path, vegetable: str
) -> Path:
    """統計表情報のJSONを取得して保存し、保存したファイルパスを返す"""
    getStatsList_response_file = Path(
        download_dir, f"getStatsList_response_{vegetable}.json"
    )
    if getStatsList_response_file.is_file():
        return getStatsList_response_file

    # 今回欲しい「青果物卸売市場調査（産地別）」には固有のIDが振られていないので文字列検索する。
    # 「かぶ」で文字列検索すると他の品目も取得されてしまうので limit=100 にして、
    # csv を保存するときに ["TITLE_SPEC"]["TABLE_NAME"] で絞り込む。
    getStatsList_results = requests.get(
        f"https://api.e-stat.go.jp/rest/3.0/app/json/getStatsList?appId={estat_app_id}&searchWord=野菜の主要消費地域別・産地別の卸売数量及び卸売価格 AND {vegetable}&statsCode=00500226&surveyYears=201801-202212&limit=100"
    )
    print(
        f"getStatsList?appId={estat_app_id}&searchWord=野菜の主要消費地域別・産地別の卸売数量及び卸売価格 AND {vegetable}&statsCode=00500226&surveyYears=201801-202212&limit=100, status code: {getStatsList_results.status_code}"
    )
    with open(getStatsList_response_file, "w") as file:
        file.write(getStatsList_results.text)
    return getStatsList_response_file


def read_response_json(file_path: Path) -> dict:
    """JSON ファイルの中身を Python のデータ型として返す"""
    file = open(file_path, "r")
    getStatsList_response_json_dict = json.load(file)
    file.close()
    return getStatsList_response_json_dict


def get_relevant_info(table_info_element) -> my_types.RelevantInfo:
    """getStatsList からののレスポンス JSON の TABLE_INF の1つの要素から必要な情報だけを抽出する。"""
    return {
        "id": table_info_element["@id"],  # 統計表ID
        "statistics_name": table_info_element["STATISTICS_NAME"],
        "title": table_info_element["TITLE"]["$"],
        "survey_date": table_info_element["SURVEY_DATE"],
        "updated_date": table_info_element["UPDATED_DATE"],
        "item_name": table_info_element["TITLE_SPEC"]["TABLE_NAME"],  # 品目
        "month": table_info_element["TITLE_SPEC"]["TABLE_SUB_CATEGORY1"],
    }
    # return の例
    # {
    #     "id": "0001872063",
    #     "statistics_name": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
    #     "title": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん １月",
    #     "survey_date": "201801-201812",
    #     "updated_date": "2021-11-10",
    #     "item_name": "だいこん",
    #     "month": "１月",
    # }


def get_list_of_relevant_info(response_json: dict) -> list[my_types.RelevantInfo]:
    """TABLE_INF のそれぞれの要素から必要な情報だけを抽出した dict の list を返す"""
    table_infos: list[my_types.TableInfo] = response_json["GET_STATS_LIST"][
        "DATALIST_INF"
    ]["TABLE_INF"]
    # table_infos の要素の例
    #
    # {
    #     "@id": "0001873393",
    #     "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
    #     "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
    #     "STATISTICS_NAME": "青果物卸売市場調査 確報 令和元年青果物卸売市場調査（産地別）",
    #     "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ５月"},
    #     "CYCLE": "年次",
    #     "SURVEY_DATE": "201901-201912",
    #     "OPEN_DATE": "2021-10-29",
    #     "SMALL_AREA": 0,
    #     "COLLECT_AREA": "該当なし",
    #     "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
    #     "SUB_CATEGORY": {"@code": "01", "$": "農業"},
    #     "OVERALL_TOTAL_NUMBER": 1152,
    #     "UPDATED_DATE": "2022-01-07",
    #     "STATISTICS_NAME_SPEC": {
    #         "TABULATION_CATEGORY": "青果物卸売市場調査",
    #         "TABULATION_SUB_CATEGORY1": "確報",
    #         "TABULATION_SUB_CATEGORY2": "令和元年青果物卸売市場調査（産地別）",
    #     },
    #     "DESCRIPTION": "",
    #     "TITLE_SPEC": {
    #         "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
    #         "TABLE_NAME": "だいこん",
    #         "TABLE_SUB_CATEGORY1": "５月",
    #     },
    # }
    return [
        get_relevant_info(table_info_element)
        for table_info_element in table_infos
        if table_info_element["TITLE_SPEC"]["TABLE_SUB_CATEGORY1"]
        != "計"  # ここの「計」はすべての月の合計であり、各月のデータの中にある品目の「計」とは異なるので注意。
    ]


def create_csv_file_name(relevant_info: my_types.RelevantInfo) -> str:
    return f"{relevant_info['id']}_{relevant_info['survey_date']}_{relevant_info['item_name']}_{relevant_info['month']}.csv"


def get_month_prefecture_total_dict(
    csv_path: Path, relevant_info: my_types.RelevantInfo
):  # -> my_types.DataWithAdditionalInfo # 型どうやるんだ？
    """ひとつの月について、各都道府県産の取り扱い数量の合計を返す。"""
    csv_df = pandas.read_csv(csv_path)
    # 数値ではない値をすべて None にする
    # https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/index.html#13
    csv_df = csv_df.replace(["-", "…", "****"], None)
    # 各行について、該当する都道府県の数値に足していく。
    prefecture = None
    all_data = {}
    for row_series in csv_df.iterrows():
        row = row_series[1]
        if row.iloc[1] != prefecture:
            # key が一定ではない（2018年は「(G001-30-2-001)都道府県」、2019年は「(G001-01-2-001)都道府県」）ので iloc で都道府県を見る。（CSV の形が違うと大変なことになる。）（2018-2020までは大丈夫っぽい。）
            prefecture = row.iloc[1]
            all_data[prefecture] = 0
        if (
            row.iloc[3] == "数量"
            and row["value"] != None
            and row.iloc[5] not in ["東北_仙台・盛岡", "関東_京浜", "九州_北九州"]
        ):
            value = int(row["value"])
            all_data[prefecture] += value

    # フロントの GeoJSON は「県」まで入っているのでそのへんを調整する。
    all_data_copy = copy.deepcopy(all_data)
    for key in all_data.keys():
        if key in ["計", "北海道"]:
            pass
        elif key in ["東京"]:
            all_data_copy[key + "都"] = all_data_copy[key]
            del all_data_copy[key]
        elif key in ["大阪", "京都"]:
            all_data_copy[key + "府"] = all_data_copy[key]
            del all_data_copy[key]
        else:
            all_data_copy[key + "県"] = all_data_copy[key]
            del all_data_copy[key]

    # その他必要な情報を付け加える。
    all_data_copy_with_additional_info = (
        {  # : my_types.DataWithAdditionalInfo # 型どうやるんだ？
            "year": relevant_info["survey_date"][0:4],
            "month": relevant_info["month"],
            "item_name": relevant_info["item_name"],
            "data": all_data_copy,  # 型どうやるんだ？
        }
    )
    return all_data_copy_with_additional_info
    # return の例
    # {
    #     "year": "2018",
    #     "month": "１月",
    #     "item_name": "だいこん",
    #     "data": {
    #         "計": 43614,
    #         "北海道": 319,
    #         "青森県": 197,
    #         ...
    #         "宮崎県": 918,
    #         "鹿児島県": 5935,
    #         "沖縄県": 0,
    #     },
    # }


def create_frontend_json(list_of_month_prefecture_total_dicts: list) -> str:
    """フロントで使う用の JSON を作成する。"""
    final_dict = {}
    for element in list_of_month_prefecture_total_dicts:
        final_dict.setdefault(element["year"], {})
        final_dict[element["year"]].setdefault(element["month"], {})
        final_dict[element["year"]][element["month"]] = element["data"]
    return json.dumps(final_dict)


def save_frontend_json(save_dir: Path, item_name: str, json_dumps: str) -> Path:
    """フロントで使う用の JSON 文字列をファイルとして保存する。"""
    file_path = Path(save_dir, f"{item_name}.json")
    if file_path.is_file():
        return file_path
    with open(file_path, "w") as file:
        file.write(json_dumps)
    return file_path


def create_and_save_frontend_json(
    relevant_info_list: list[my_types.RelevantInfo],
    vegetable: str,
    vegetable_csv_download_dir: Path,
    list_of_month_prefecture_total_dicts: list,
    processed_data_dir: Path,
):
    """1つの野菜についての、月ごとの合計をまとめたJSONを作る"""
    for relevant_info in relevant_info_list:
        # 「かぶ」で文字列検索すると他の品目も relevant_info_list に入ってしまうので ["TITLE_SPEC"]["TABLE_NAME"] が現ループの品目と一致するかを確認する。
        # 「かぶ」以外では必要ないけどあっても損はないのですべての品目で走るようにしている。
        if relevant_info["item_name"] != vegetable:
            continue

        one_month_csv = estat_utils.download_stats_data_as_CSV(
            download_dir=vegetable_csv_download_dir,
            file_name=create_csv_file_name(relevant_info),
            estat_app_id=ESTAT_APP_ID,
            stats_data_id=relevant_info["id"],
            section_header=False,
        )
        print(one_month_csv)
        list_of_month_prefecture_total_dicts.append(
            get_month_prefecture_total_dict(
                csv_path=one_month_csv, relevant_info=relevant_info
            )
        )

    frontend_json = create_frontend_json(
        list_of_month_prefecture_total_dicts=list_of_month_prefecture_total_dicts
    )
    save_frontend_json(
        save_dir=processed_data_dir, item_name=vegetable, json_dumps=frontend_json
    )
