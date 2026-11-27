import unittest

from ledgerlite.money import parse_amount


class ParseAmount(unittest.TestCase):
    def test_plain(self):
        self.assertEqual(parse_amount("5"), 500)
        self.assertEqual(parse_amount("5.5"), 550)
        self.assertEqual(parse_amount("5.05"), 505)

    def test_dollar_and_whitespace(self):
        self.assertEqual(parse_amount("  $12.30 "), 1230)

    def test_negative_forms(self):
        self.assertEqual(parse_amount("-12.50"), -1250)
        self.assertEqual(parse_amount("(12.50)"), -1250)
        self.assertEqual(parse_amount("-$3.10"), -310)

    def test_thousands(self):
        self.assertEqual(parse_amount("1,234,567.89"), 123456789)

    def test_bad_grouping(self):
        for bad in ("1,23.00", "12,34"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                parse_amount(bad)

    def test_rejects_three_decimals(self):
        with self.assertRaises(ValueError):
            parse_amount("2.999")

    def test_three_decimal_input_rounds(self):
        self.assertEqual(parse_amount("1.005"), 101)

    def test_rejects_junk(self):
        for bad in ("", "-", "$", ".5", "12a", "1.2.3"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                parse_amount(bad)


if __name__ == "__main__":
    unittest.main()
