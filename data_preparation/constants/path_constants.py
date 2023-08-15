import os
from pathlib import Path

# パスがめんどくさくなってきたのでここで定数として定義しておく

current_file_location = Path(os.path.dirname(os.path.realpath(__file__)))

# processes
processes_path = Path(os.path.realpath(Path(current_file_location, "..", "processes")))

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
frontend_path = Path(
    os.path.realpath(Path(current_file_location, "..", "..", "frontend"))
)
frontend_assets_path = Path(os.path.realpath(Path(frontend_path, "public", "assets")))
