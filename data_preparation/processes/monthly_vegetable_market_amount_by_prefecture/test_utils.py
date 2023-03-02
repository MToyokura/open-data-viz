import shutil
import unittest
from pathlib import Path

import processes.monthly_vegetable_market_amount_by_prefecture.utils as utils
from processes.estat.variables import ESTAT_APP_ID
from processes.monthly_vegetable_market_amount_by_prefecture import my_types

sample_dir = Path("./sample_dir/")


class TestFetchingForGetStatsListAndSavingJSON(unittest.TestCase):
    def setUp(self) -> None:
        sample_dir.mkdir()

    def test_get_and_save_json_response(self) -> None:
        "統計表情報のJSONを取得、保存して中身を確認する。"
        json_file_path = utils.get_and_save_json_response(
            ESTAT_APP_ID, sample_dir, "だいこん"
        )
        self.assertEqual(
            json_file_path, Path(sample_dir, "getStatsList_response_だいこん.json")
        )

        # 中身をチェック
        response_json = utils.read_response_json(
            Path(sample_dir, "getStatsList_response_だいこん.json")
        )
        self.assertEqual(
            response_json["GET_STATS_LIST"]["RESULT"]["ERROR_MSG"], "正常に終了しました。"
        )

    def tearDown(self) -> None:
        shutil.rmtree(sample_dir)


class TestGetListOfRelevantInfo(unittest.TestCase):
    def setUp(self) -> None:
        sample_dir.mkdir()

    def test_get_list_of_relevant_info(self):
        "統計表情報のJSONから必要な情報を抜き出せているかを確認する。"
        list_of_relevant_info = utils.get_list_of_relevant_info(
            dummy_response_json_dict
        )
        self.assertEqual(
            # 適当に4つ目を指定してみる
            list_of_relevant_info[3]["id"],
            "0001872066",
        )

    def test_month_prefecture_total(self):
        "1ヶ月の都道府県ごとの合計値を計算できていることを確認する。"

        actual_month_prefecture_total = utils.get_month_prefecture_total_dict(
            Path(
                "processes/monthly_vegetable_market_amount_by_prefecture/for_test_0001873398_201901-201912_だいこん_10月.csv"
            ),
            dummy_relevant_info,
        )
        self.assertDictEqual(
            actual_month_prefecture_total, dummy_month_prefecture_total
        )

    def tearDown(self) -> None:
        shutil.rmtree(sample_dir)


dummy_response_json_dict = {
    "GET_STATS_LIST": {
        "RESULT": {
            "STATUS": 0,
            "ERROR_MSG": "正常に終了しました。",
            "DATE": "2022-11-15T14:37:01.650+09:00",
        },
        "PARAMETER": {
            "LANG": "J",
            "SURVEY_YEARS": "201801-202212",
            "STATS_CODE": "00500226",
            "SEARCH_WORD": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 AND だいこん",
            "DATA_FORMAT": "J",
            "LIMIT": 100,
        },
        "DATALIST_INF": {
            "NUMBER": 39,
            "RESULT_INF": {"FROM_NUMBER": 1, "TO_NUMBER": 39},
            "TABLE_INF": [
                {
                    "@id": "0001872062",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 計"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "計",
                    },
                },
                {
                    "@id": "0001872063",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん １月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "１月",
                    },
                },
                {
                    "@id": "0001872064",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ２月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "２月",
                    },
                },
                {
                    "@id": "0001872065",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ３月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "３月",
                    },
                },
                {
                    "@id": "0001872066",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ４月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "４月",
                    },
                },
                {
                    "@id": "0001872067",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ５月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "５月",
                    },
                },
                {
                    "@id": "0001872068",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ６月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "６月",
                    },
                },
                {
                    "@id": "0001872069",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ７月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "７月",
                    },
                },
                {
                    "@id": "0001872070",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ８月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "８月",
                    },
                },
                {
                    "@id": "0001872071",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん ９月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "９月",
                    },
                },
                {
                    "@id": "0001872072",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 10月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "10月",
                    },
                },
                {
                    "@id": "0001872073",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 11月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "11月",
                    },
                },
                {
                    "@id": "0001872074",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 平成30年青果物卸売市場調査報告（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 12月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201801-201812",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2021-11-10",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "平成30年青果物卸売市場調査報告（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "12月",
                    },
                },
                {
                    "@id": "0001873388",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 令和元年青果物卸売市場調査（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 計"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201901-201912",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2022-01-07",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "令和元年青果物卸売市場調査（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "計",
                    },
                },
                {
                    "@id": "0001873389",
                    "STAT_NAME": {"@code": "00500226", "$": "青果物卸売市場調査"},
                    "GOV_ORG": {"@code": "00500", "$": "農林水産省"},
                    "STATISTICS_NAME": "青果物卸売市場調査 確報 令和元年青果物卸売市場調査（産地別）",
                    "TITLE": {"@no": "1-1", "$": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん １月"},
                    "CYCLE": "年次",
                    "SURVEY_DATE": "201901-201912",
                    "OPEN_DATE": "2021-10-29",
                    "SMALL_AREA": 0,
                    "COLLECT_AREA": "該当なし",
                    "MAIN_CATEGORY": {"@code": "04", "$": "農林水産業"},
                    "SUB_CATEGORY": {"@code": "01", "$": "農業"},
                    "OVERALL_TOTAL_NUMBER": 1152,
                    "UPDATED_DATE": "2022-01-07",
                    "STATISTICS_NAME_SPEC": {
                        "TABULATION_CATEGORY": "青果物卸売市場調査",
                        "TABULATION_SUB_CATEGORY1": "確報",
                        "TABULATION_SUB_CATEGORY2": "令和元年青果物卸売市場調査（産地別）",
                    },
                    "DESCRIPTION": "",
                    "TITLE_SPEC": {
                        "TABLE_CATEGORY": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
                        "TABLE_NAME": "だいこん",
                        "TABLE_SUB_CATEGORY1": "１月",
                    },
                },
            ],
        },
    }
}

dummy_relevant_info: my_types.RelevantInfo = {
    "id": "0001873398",
    "statistics_name": "青果物卸売市場調査 確報 令和元年青果物卸売市場調査（産地別）",
    "title": "野菜の主要消費地域別・産地別の卸売数量及び卸売価格 だいこん 10月",
    "survey_date": "201901-201912",
    "updated_date": "2022-01-07",
    "item_name": "だいこん",
    "month": "10月",
}

dummy_month_prefecture_total = {
    "year": "2019",
    "month": "10月",
    "item_name": "だいこん",
    "data": {
        "計": 67896,
        "北海道": 20629,
        "青森県": 21655,
        "岩手県": 3257,
        "宮城県": 130,
        "秋田県": 60,
        "山形県": 199,
        "福島県": 32,
        "茨城県": 1811,
        "栃木県": 813,
        "群馬県": 1325,
        "埼玉県": 234,
        "千葉県": 3640,
        "東京都": 56,
        "神奈川県": 154,
        "新潟県": 2492,
        "富山県": 139,
        "石川県": 3882,
        "福井県": 146,
        "山梨県": 14,
        "長野県": 326,
        "岐阜県": 960,
        "静岡県": 407,
        "愛知県": 253,
        "三重県": 7,
        "滋賀県": 105,
        "京都府": 18,
        "大阪府": 7,
        "兵庫県": 150,
        "奈良県": 2,
        "和歌山県": 6,
        "鳥取県": 103,
        "島根県": 17,
        "岡山県": 475,
        "広島県": 263,
        "山口県": 421,
        "徳島県": 140,
        "香川県": 6,
        "愛媛県": 26,
        "高知県": 37,
        "福岡県": 73,
        "佐賀県": 0,
        "長崎県": 533,
        "熊本県": 1744,
        "大分県": 220,
        "宮崎県": 141,
        "鹿児島県": 782,
        "沖縄県": 0,
    },
}
