# 東京都の離島などを省略するスクリプト

import json
import sys
from pathlib import Path

input_file_path = Path(sys.argv[1])
output_file_path = Path(sys.argv[2])

with open(input_file_path, "r") as json_file:
    data = json.load(json_file)


def omit_polygons(
    coordinates: list[list[list[float]]], threshold: int
) -> list[list[list[float]]]:
    """threshold 以下の辺の数のポリゴンを除く"""
    if len(coordinates) == 1:
        return coordinates
    return_coordinates = []
    for coordinate in coordinates:
        for lat_long_pairs in coordinate:
            if len(lat_long_pairs) > threshold:
                return_coordinates.append(coordinate)
    return return_coordinates


for feature in data["features"]:
    feature["geometry"]["coordinates"] = omit_polygons(
        feature["geometry"]["coordinates"], 10
    )
    if feature["properties"]["ADM1_JA"] == "鹿児島県":
        # 加計呂麻島と奄美大島を除く
        del feature["geometry"]["coordinates"][0:2]


with open(output_file_path, "w") as file:
    file.write(json.dumps(data))
