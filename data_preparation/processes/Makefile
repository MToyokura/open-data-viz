# 参考文献
# https://bost.ocks.org/mike/make/
# https://makefiletutorial.com/
# https://superuser.com/a/370576


# 全体で共通する変数
FRONTEND_DIR := ../../frontend/public/assets
RAW_DATA_DIR := ../raw_data
PROCESSED_DATA_DIR := ../processed_data

all: make_initial_directories prefectures make_monthly_vegetable_market_amount_by_prefecture files_list make_workers_by_age_employment_type_education_sex_files

# raw_data/ と processed_data/ のファイル一覧を作成する。
files_list:
	ls -R $(RAW_DATA_DIR) > $(RAW_DATA_DIR)/files_list.txt
	ls -R $(PROCESSED_DATA_DIR) > $(PROCESSED_DATA_DIR)/files_list.txt

# 労働力調査 詳細集計　全都道府県 全国 年次  年齢階級，教育，雇用形態別雇用者数(2002年～) のデータを作成する
make_workers_by_age_employment_type_education_sex_files:
	make -C ./workers_by_age_employment_type_education_sex

# 品目ごとの月別の卸売市場取扱量のデータを作成する。
make_monthly_vegetable_market_amount_by_prefecture:
	make -C ./monthly_vegetable_market_amount_by_prefecture

# 都道府県の GeoJSON を生成する。
prefectures:
	make -C ./prefecture_map

# 必要なディレクトリを作成
make_initial_directories:
	mkdir -p $(FRONTEND_DIR)
	mkdir -p $(RAW_DATA_DIR)
	mkdir -p $(PROCESSED_DATA_DIR)