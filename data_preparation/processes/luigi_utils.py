from pathlib import Path


def initialize_data_folder(folder_name: Path):
    """Initialize folder for data preparation.

    Args:
        folder_name (str): Folder name to initialize.

    Returns:
        None
    """
    if not Path(folder_name, "raw_data").exists():
        Path(folder_name, "raw_data").mkdir()
    if not Path(folder_name, "intermediate_data").exists():
        Path(folder_name, "intermediate_data").mkdir()
    if not Path(folder_name, "final_data").exists():
        Path(folder_name, "final_data").mkdir()
