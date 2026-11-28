import unittest

from billable.rounding import round_up


class RoundUpTest(unittest.TestCase):
    def test_zero_stays_zero(self):
        self.assertEqual(round_up(0), 0)

    def test_one_minute_bills_a_full_increment(self):
        self.assertEqual(round_up(1), 6)

    def test_exact_multiple_is_unchanged(self):
        self.assertEqual(round_up(6), 6)
        self.assertEqual(round_up(48), 48)

    def test_just_past_a_multiple(self):
        self.assertEqual(round_up(7), 12)

    def test_quarter_hour_increment(self):
        self.assertEqual(round_up(16, 15), 30)
        self.assertEqual(round_up(15, 15), 15)

    def test_increment_of_one(self):
        self.assertEqual(round_up(37, 1), 37)

    def test_rejects_zero_increment(self):
        with self.assertRaises(ValueError):
            round_up(5, 0)

    def test_rejects_negative_minutes(self):
        with self.assertRaises(ValueError):
            round_up(-1)
