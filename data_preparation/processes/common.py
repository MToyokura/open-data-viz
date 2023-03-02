from pathlib import Path

raw_data_dir_base = Path("../../raw_data/")
processed_data_dir_base = Path("../../processed_data/")


def create_raw_data_dir_base_path(data_dir: str):
    return Path(raw_data_dir_base, data_dir)


def create_processed_data_dir_base_path(data_dir: str):
    return Path(processed_data_dir_base, data_dir)
