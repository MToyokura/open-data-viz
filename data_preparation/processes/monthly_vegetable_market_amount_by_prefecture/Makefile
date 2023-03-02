# 全体で共通する変数
FRONTEND_DIR := ../../../frontend/public/assets
RAW_DATA_DIR := ../../raw_data
PROCESSED_DATA_DIR := ../../processed_data

all: make_monthly_vegetable_market_amount_by_prefecture export_to_frontend

export_to_frontend:
	cp -r $(PROCESSED_DATA_DIR)/monthly_vegetable_market_amount_by_prefecture $(FRONTEND_DIR)

make_monthly_vegetable_market_amount_by_prefecture:
	poetry run python download_from_estat.py