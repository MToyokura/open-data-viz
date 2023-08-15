import unittest

import constants.path_constants


class TestPathConstants(unittest.TestCase):
    def test_constants_path(self):
        """constants ディレクトリのパスが変わっていないかを確認する"""
        self.assertTrue(
            "open-data-viz/data_preparation/constants"
            in str(constants.path_constants.path_constants_location)
        )

    def test_processes_directory_structure(self):
        """processes ディレクトリの構造が変わっていないかを確認する"""
        self.assertTrue(
            "open-data-viz/data_preparation/processes/commodity_prices"
            in str(constants.path_constants.commodity_prices_path)
        )
        self.assertTrue(
            "open-data-viz/data_preparation/processes/estat"
            in str(constants.path_constants.estat_path)
        )
        self.assertTrue(
            "open-data-viz/data_preparation/processes/monthly_vegetable_market_amount_by_prefecture"
            in str(
                constants.path_constants.monthly_vegetable_market_amount_by_prefecture_path
            )
        )
        self.assertTrue(
            "open-data-viz/data_preparation/processes/prefecture_map"
            in str(constants.path_constants.prefecture_map_path)
        )
        self.assertTrue(
            "open-data-viz/data_preparation/processes/workers_by_age_employment_type_education_sex"
            in str(
                constants.path_constants.workers_by_age_employment_type_education_sex_path
            )
        )

    def test_frontend_directory_structure(self):
        """frontend ディレクトリの構造が変わっていないかを確認する"""
        self.assertTrue(
            "open-data-viz/frontend" in str(constants.path_constants.frontend_path)
        )
        self.assertTrue(
            "open-data-viz/frontend/public/assets"
            in str(constants.path_constants.frontend_assets_path)
        )
