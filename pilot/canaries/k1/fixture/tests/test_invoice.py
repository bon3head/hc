import unittest

from billable.invoice import bill, overtime


class BillTest(unittest.TestCase):
    def test_empty(self):
        self.assertEqual(bill([], 9000), 0)

    def test_one_increment(self):
        self.assertEqual(bill(["6m"], 6000), 600)

    def test_short_entry_rounds_up(self):
        self.assertEqual(bill(["1m"], 6000), 600)

    def test_entries_round_separately(self):
        # 7 -> 12 and 5 -> 6, not 12 -> 12
        self.assertEqual(bill(["7m", "5m"], 6000), 1800)

    def test_half_cent_rounds_up(self):
        # 6 minutes at 10.05/h is 100.5 cents
        self.assertEqual(bill(["6m"], 1005), 101)

    def test_below_half_cent_rounds_down(self):
        self.assertEqual(bill(["5m"], 1001), 100)

    def test_whole_hour(self):
        self.assertEqual(bill(["1h"], 12345), 12345)

    def test_custom_increment(self):
        self.assertEqual(bill(["7m"], 6000, increment=1), 700)
        self.assertEqual(bill(["7m"], 6000, increment=15), 1500)

    def test_mixed_week(self):
        self.assertEqual(bill(["7m", "50m", "1:13", "2h"], 7999), 35329)

    def test_invalid_entry_raises(self):
        with self.assertRaises(ValueError):
            bill(["2h", "soon"], 9000)


class OvertimeTest(unittest.TestCase):
    def test_standard_week(self):
        self.assertEqual(overtime([480] * 5), 0)

    def test_long_days(self):
        self.assertEqual(overtime([540] * 5), 300)

    def test_exactly_forty_hours(self):
        self.assertEqual(overtime([600] * 4), 0)

    def test_short_day_after_forty(self):
        self.assertEqual(overtime([600] * 4 + [30]), 30)
