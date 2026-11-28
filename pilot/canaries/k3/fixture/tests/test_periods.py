import unittest
from datetime import date

from fiscal.calendar import fiscal_period, period_bounds


class PeriodTest(unittest.TestCase):
    def test_first_period_is_four_weeks(self):
        self.assertEqual(period_bounds(2022, 1), (date(2022, 1, 30), date(2022, 2, 26)))

    def test_third_period_is_five_weeks(self):
        start, end = period_bounds(2022, 3)
        self.assertEqual(((end - start).days + 1) // 7, 5)

    def test_long_year_puts_the_extra_week_last(self):
        self.assertEqual(period_bounds(2023, 1), (date(2023, 1, 29), date(2023, 2, 25)))
        self.assertEqual(period_bounds(2023, 12), (date(2023, 12, 24), date(2024, 2, 3)))

    def test_period_of_a_date(self):
        self.assertEqual(fiscal_period(date(2024, 1, 30)), (2023, 12))
        self.assertEqual(fiscal_period(date(2025, 3, 1)), (2025, 1))

    def test_periods_tile_the_year(self):
        for fy in (2022, 2023, 2028):
            with self.subTest(fy=fy):
                prev_end = None
                for p in range(1, 13):
                    start, end = period_bounds(fy, p)
                    if prev_end:
                        self.assertEqual((start - prev_end).days, 1)
                    prev_end = end

    def test_rejects_bad_period(self):
        with self.assertRaises(ValueError):
            period_bounds(2024, 13)
