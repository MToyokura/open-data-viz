"""Unit checks for the deliberately provisional app-data exporter."""
import unittest
from export_2024_daikon_demo import aggregate, number


class DemoExportTests(unittest.TestCase):
    def test_missing_not_silently_zero(self):
        result = aggregate([
            {"quantity": {"kind": "reported", "tonnes_lexical": "1,234.5"}},
            {"quantity": {"kind": "below_display_unit"}},
            {"quantity": {"kind": "no_fact"}},
        ])
        self.assertEqual(result["reported_tonnes_sum"], "1234.5")
        self.assertEqual(result["completeness"], "partial")
        self.assertEqual(result["below_display_unit_cells"], 1)
        self.assertEqual(result["no_fact_cells"], 1)

    def test_all_reported(self):
        self.assertEqual(aggregate([{"quantity": {"kind": "reported", "tonnes_lexical": "3"}}])["completeness"], "all_reported")

    def test_invalid_numeric_rejected(self):
        for value in ("0", "-1", "NaN", "1,23", "1e3"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                number(value)


if __name__ == "__main__":
    unittest.main()
