import unittest
from datetime import date

from fiscal.calendar import fiscal_week


class WeekTest(unittest.TestCase):
    def test_first_day_is_week_one(self):
        self.assertEqual(fiscal_week(date(2022, 1, 30)), (2022, 1))

    def test_saturday_closes_the_week(self):
        self.assertEqual(fiscal_week(date(2022, 2, 5)), (2022, 1))
        self.assertEqual(fiscal_week(date(2022, 2, 6)), (2022, 2))

    def test_last_week_of_a_long_year(self):
        self.assertEqual(fiscal_week(date(2024, 2, 2)), (2023, 53))

    def test_early_february_can_belong_to_the_previous_year(self):
        self.assertEqual(fiscal_week(date(2025, 2, 1)), (2024, 52))

    def test_mid_year(self):
        self.assertEqual(fiscal_week(date(2025, 7, 4)), (2025, 22))
