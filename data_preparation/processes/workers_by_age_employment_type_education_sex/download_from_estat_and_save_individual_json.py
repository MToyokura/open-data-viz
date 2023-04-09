import os
from pathlib import Path

import pandas
import processes.estat.utils as estat_utils
from processes.estat.variables import ESTAT_APP_ID
from processes.workers_by_age_employment_type_education_sex import utils

# estat からデータをダウンロードして、フロントエンド向けにファイルを作成したりする

current_file_location = Path(os.path.dirname(os.path.realpath(__file__)))
files_dir_name = "workers_by_age_employment_type_education_sex_files"
raw_data_dir = Path(current_file_location, "raw_data")
final_data_dir = Path(current_file_location, "final_data")

statsDataId = "0003006608"
# https://www.e-stat.go.jp/dbview?sid=0003006608
# 労働力調査の概要，結果等
# https://www.stat.go.jp/data/roudou/index2.html
# 労働力調査　用語の解説
# https://www.stat.go.jp/data/roudou/definit.html
# 労働力調査 調査結果利用上の注意
# https://www.stat.go.jp/data/roudou/chuui.html
downloaded_csv_path = estat_utils.download_stats_data_as_CSV(
    download_dir=raw_data_dir,
    file_name=f"{statsDataId}_年齢階級，教育，雇用形態別雇用者数(2002年～).csv",
    estat_app_id=ESTAT_APP_ID,
    stats_data_id=f"{statsDataId}",
    section_header=False,
)

csv_df = pandas.read_csv(downloaded_csv_path).replace(["…"], None)  # 2011年が "…"

utils.create_json_files(csv_df=csv_df, processed_data_dir=final_data_dir)
