import unittest
from pathlib import Path

import processes.common


class DataDirPathCreationTest(unittest.TestCase):
    def test_create_raw_data_dir_base_path(self) -> None:
        data_dir = "foobar"
        returned_path = processes.common.create_raw_data_dir_base_path(data_dir)
        self.assertEqual(returned_path, Path("../../raw_data/foobar"))

    def test_create_processed_data_dir_base_path(self) -> None:
        data_dir = "foobar"
        returned_path = processes.common.create_processed_data_dir_base_path(data_dir)
        self.assertEqual(returned_path, Path("../../processed_data/foobar"))
