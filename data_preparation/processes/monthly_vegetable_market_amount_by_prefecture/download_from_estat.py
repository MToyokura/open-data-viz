import os
from pathlib import Path

import utils
from processes.common import (
    create_processed_data_dir_base_path,
    create_raw_data_dir_base_path,
)
from processes.estat.variables import ESTAT_APP_ID

files_dir_name = "monthly_vegetable_market_amount_by_prefecture"
raw_data_dir = create_raw_data_dir_base_path(files_dir_name)
processed_data_dir = create_processed_data_dir_base_path(files_dir_name)
raw_data_dir.mkdir(parents=True, exist_ok=True)
processed_data_dir.mkdir(parents=True, exist_ok=True)

# https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/index.html#7
vegetables = [
    "だいこん",
    "かぶ",
    "にんじん",
    "ごぼう",
    "たけのこ",
    "れんこん",
    "はくさい",
    "みずな",
    "こまつな",
    "その他の菜類",
    "ちんげんさい",
    "キャベツ",
    "ほうれんそう",
    "ねぎ",
    "ふき",
    "うど",
    "みつば",
    "しゅんぎく",
    "にら",
    "セルリー",
    "アスパラガス",
    "カリフラワー",
    "ブロッコリー",
    "レタス",
    "パセリ",
    "きゅうり",
    "かぼちゃ",
    "なす",
    "トマト",
    "ミニトマト",
    "ピーマン",
    "ししとうがらし",
    "スイートコーン",
    "さやいんげん",
    "さやえんどう",
    "実えんどう",
    "そらまめ",
    "えだまめ",
    "かんしょ",
    "ばれいしょ",
    "さといも",
    "やまのいも",
    "たまねぎ",
    "にんにく",
    "しょうが",
    "生しいたけ",
    "なめこ",
    "えのきだけ",
    "しめじ",
    "その他の野菜",
]

for vegetable in vegetables:
    json_response_file = utils.get_and_save_json_response(
        estat_app_id=ESTAT_APP_ID, download_dir=raw_data_dir, vegetable=vegetable
    )
    print(json_response_file)

    getStatsList_response_json_dict = utils.read_response_json(json_response_file)
    relevant_info_list = utils.get_list_of_relevant_info(
        getStatsList_response_json_dict
    )

    frontend_json_file_path = Path(processed_data_dir, f"{vegetable}.json")
    if not frontend_json_file_path.is_file():
        list_of_month_prefecture_total_dicts = []
        vegetable_csv_download_dir = Path(raw_data_dir, vegetable)
        vegetable_csv_download_dir.mkdir(parents=True, exist_ok=True)
        utils.create_and_save_frontend_json(
            relevant_info_list=relevant_info_list,
            vegetable=vegetable,
            vegetable_csv_download_dir=vegetable_csv_download_dir,
            list_of_month_prefecture_total_dicts=list_of_month_prefecture_total_dicts,
            processed_data_dir=processed_data_dir,
        )
    print(frontend_json_file_path)
