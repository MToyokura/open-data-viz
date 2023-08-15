import os
from pathlib import Path

# システム上の絶対パスを定数として定義しておく

# このファイルの絶対パス
path_constants_location = Path(os.path.dirname(os.path.realpath(__file__)))

# root (open-data-viz)
root_path = Path(os.path.realpath(Path(path_constants_location, "..", "..")))

# data_preparation
data_preparation_path = Path(os.path.realpath(Path(root_path, "data_preparation")))

# processes
processes_path = Path(os.path.realpath(Path(data_preparation_path, "processes")))

# processes/*
commodity_prices_path = Path(os.path.realpath(Path(processes_path, "commodity_prices")))
estat_path = Path(os.path.realpath(Path(processes_path, "estat")))
monthly_vegetable_market_amount_by_prefecture_path = Path(
    os.path.realpath(
        Path(processes_path, "monthly_vegetable_market_amount_by_prefecture")
    )
)
prefecture_map_path = Path(os.path.realpath(Path(processes_path, "prefecture_map")))
workers_by_age_employment_type_education_sex_path = Path(
    os.path.realpath(
        Path(processes_path, "workers_by_age_employment_type_education_sex")
    )
)

# frontend
frontend_path = Path(os.path.realpath(Path(root_path, "frontend")))
frontend_assets_path = Path(os.path.realpath(Path(frontend_path, "public", "assets")))
