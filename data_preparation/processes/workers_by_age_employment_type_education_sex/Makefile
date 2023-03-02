# 全体で共通する変数
FRONTEND_DIR := ../../../frontend/public/assets
RAW_DATA_DIR := ../../raw_data
PROCESSED_DATA_DIR := ../../processed_data

all: make_workers_by_age_employment_type_education_sex_files export_to_frontend

export_to_frontend:
	cp -r $(PROCESSED_DATA_DIR)/workers_by_age_employment_type_education_sex_files $(FRONTEND_DIR)

make_workers_by_age_employment_type_education_sex_files:
	poetry run python download_from_estat_and_save_individual_json.py