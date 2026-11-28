import unittest

from billable.durations import format_duration, parse_duration


class ParseDurationTest(unittest.TestCase):
    def test_minutes_only(self):
        self.assertEqual(parse_duration("45m"), 45)

    def test_hours_only(self):
        self.assertEqual(parse_duration("2h"), 120)

    def test_hours_and_minutes(self):
        self.assertEqual(parse_duration("2h15m"), 135)

    def test_clock_form(self):
        self.assertEqual(parse_duration("1:05"), 65)

    def test_whitespace_and_case(self):
        self.assertEqual(parse_duration("  2H15M "), 135)

    def test_minutes_part_out_of_range(self):
        with self.assertRaises(ValueError):
            parse_duration("1h60m")

    def test_clock_minutes_out_of_range(self):
        with self.assertRaises(ValueError):
            parse_duration("1:60")

    def test_clock_needs_two_minute_digits(self):
        with self.assertRaises(ValueError):
            parse_duration("1:5")

    def test_rejects_empty_and_bare_units(self):
        for text in ["", "   ", "h", "m", "hm"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_duration(text)

    def test_rejects_signs_and_decimals(self):
        for text in ["-5m", "+5m", "1.5h", "2h 15m", "five"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_duration(text)


class FormatDurationTest(unittest.TestCase):
    def test_pads_minutes(self):
        self.assertEqual(format_duration(65), "1:05")

    def test_zero(self):
        self.assertEqual(format_duration(0), "0:00")

    def test_hours_unpadded(self):
        self.assertEqual(format_duration(600), "10:00")

    def test_round_trip(self):
        for text in ["0:07", "3:59", "12:30"]:
            with self.subTest(text=text):
                self.assertEqual(format_duration(parse_duration(text)), text)

    def test_negative_raises(self):
        with self.assertRaises(ValueError):
            format_duration(-1)
