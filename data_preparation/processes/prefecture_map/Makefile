# 元データ
# https://data.humdata.org/dataset/cod-ab-jpn
# https://data.humdata.org/dataset/6ba099c6-350b-4711-9a65-d85a1c5e519c/resource/f82faadf-a608-42cf-ae15-75ce672d7e69/download/jpn_adm_2019_shp.zip
# Contributor: OCHA Regional Office for Asia and the Pacific (ROAP) 
# The original shapefile is licensed under a Creative Commons Attribution for Intergovernmental Organisations license.

# 参考文献
# https://qiita.com/cieloazul310/items/9d0e3f67f60d791726dc
# https://curl.se/docs/manpage.html#-L
# https://stackoverflow.com/questions/9679932/how-to-use-executables-from-a-package-installed-locally-in-node-modules
# https://github.com/mbostock/shapefile/blob/master/README.md#shp2json
# https://github.com/topojson/topojson-server
# https://github.com/topojson/topojson-simplify
# https://github.com/topojson/topojson-client
# https://bost.ocks.org/mike/make/
# https://makefiletutorial.com/

# 備考
# yarn だとファイルパス指定で変な動きをするので npx を使う


# 全体で共通する変数
FRONTEND_DIR := ../../../frontend/public/assets
RAW_DATA_DIR := ../../raw_data
PROCESSED_DATA_DIR := ../../processed_data


# 47都道府県の GeoJSON を作成する。
shapefile_zip_path := $(RAW_DATA_DIR)/jpn_adm_2019_shp.zip
shapefile_path := $(RAW_DATA_DIR)/jpn_adm_2019_shp/jpn_admbnda_adm1_2019.shp
geojson_path := $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp/prefectures.geojson
topojson_path := $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp/prefectures_topo.json
simplified_topojson_path := $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp/prefectures_topo_simplified.json
simplified_geojson_path := $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp/prefectures_simplified.geojson
simplified_geojson_islands_omitted_path := $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp/simplified_geojson_island_omitted.geojson


# フロント側にエクスポート
$(FRONTEND_DIR)/simplified_geojson_island_omitted.geojson: $(simplified_geojson_islands_omitted_path)
	cp $(simplified_geojson_islands_omitted_path) $(FRONTEND_DIR)

# 離島などを省略する
$(simplified_geojson_islands_omitted_path): $(simplified_geojson_path)
	poetry run python omit_islands.py $(simplified_geojson_path) $(simplified_geojson_islands_omitted_path)

# Simplified GeoJSON
$(simplified_geojson_path): $(simplified_topojson_path)
	npx topo2geo japan=$(simplified_geojson_path) < $(simplified_topojson_path)

# Simplify
$(simplified_topojson_path): $(topojson_path)
	npx toposimplify -P 0.05 -f $(topojson_path) > $(simplified_topojson_path)

# TopoJSON に変換
$(topojson_path): $(geojson_path)
	npx geo2topo -q 1e6 japan=$(geojson_path) > $(topojson_path)

# shapefile から GeoJSON を作成する。
$(geojson_path): $(shapefile_path)
	mkdir -p $(PROCESSED_DATA_DIR)/jpn_adm_2019_shp
	npx shp2json --encoding "UTF8" $(shapefile_path) -o $(geojson_path)  

# Unzip
$(shapefile_path): $(shapefile_zip_path)
	mkdir -p $(RAW_DATA_DIR)/jpn_adm_2019_shp
	unzip $(shapefile_zip_path) -d $(RAW_DATA_DIR)/jpn_adm_2019_shp
	touch $(shapefile_path)

# shapefile の Zip をダウンロードする
$(shapefile_zip_path):
	curl -Lo $(shapefile_zip_path) "https://data.humdata.org/dataset/6ba099c6-350b-4711-9a65-d85a1c5e519c/resource/f82faadf-a608-42cf-ae15-75ce672d7e69/download/jpn_adm_2019_shp.zip"
