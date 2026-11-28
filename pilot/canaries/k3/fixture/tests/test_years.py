import unittest
from datetime import date

from fiscal.calendar import fiscal_year_end, fiscal_year_start, weeks_in_year


class YearTest(unittest.TestCase):
    def test_year_end_in_january(self):
        self.assertEqual(fiscal_year_end(2022), date(2023, 1, 28))

    def test_year_end_can_fall_in_february(self):
        self.assertEqual(fiscal_year_end(2024), date(2025, 2, 1))

    def test_year_end_on_january_31(self):
        self.assertEqual(fiscal_year_end(2025), date(2026, 1, 31))

    def test_year_start_follows_previous_end(self):
        self.assertEqual(fiscal_year_start(2025), date(2025, 2, 2))

    def test_ordinary_year_has_52_weeks(self):
        self.assertEqual(weeks_in_year(2022), 52)

    def test_long_year_has_53_weeks(self):
        self.assertEqual(weeks_in_year(2023), 53)

    def test_years_tile_without_gaps(self):
        for fy in range(2015, 2035):
            with self.subTest(fy=fy):
                self.assertEqual((fiscal_year_start(fy + 1) - fiscal_year_end(fy)).days, 1)
