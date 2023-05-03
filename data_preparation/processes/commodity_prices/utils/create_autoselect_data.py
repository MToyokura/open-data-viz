import json
from pathlib import Path


def create_autoselect_data(metadata_json_path: Path, output_json_directory: Path):
    with open(metadata_json_path, "r") as metadata_json_file:
        metadata_json = json.load(metadata_json_file)
        commodity_selections = []
        commodity_class = metadata_json["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"][
            "CLASS_OBJ"
        ][2]["CLASS"]
        for commodity in commodity_class:
            commodity_selections.append(
                {
                    "label": commodity["@name"],
                    "code": commodity["@code"],
                }
            )
        with open(
            Path(output_json_directory, "commodity_selections.json"), "w"
        ) as commodity_selections_file:
            json.dump(
                commodity_selections, commodity_selections_file, ensure_ascii=False
            )

        region_selections = []
        region_class = metadata_json["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"][
            "CLASS_OBJ"
        ][3]["CLASS"]
        for region in region_class:
            region_selections.append(
                {
                    "label": region["@name"],
                    "code": region["@code"],
                }
            )
        with open(
            Path(output_json_directory, "region_selections.json"), "w"
        ) as region_selections_file:
            json.dump(region_selections, region_selections_file, ensure_ascii=False)

    return output_json_directory
