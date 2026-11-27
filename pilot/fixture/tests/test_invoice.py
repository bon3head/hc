import unittest

from ledgerlite.invoice import invoice_total, render_tags


class InvoiceTotal(unittest.TestCase):
    def test_no_tax(self):
        self.assertEqual(invoice_total([(2, 500), (1, 250)], 0), 1250)

    def test_tax(self):
        # 1000 * 825 / 10000 = 82.5 -> 82 (half-even)
        self.assertEqual(invoice_total([(1, 1000)], 825), 1082)

    def test_tax_half_even(self):
        # 50 * 1000 / 10000 = 5.0; 25 * 1000 / 10000 = 2.5 -> 2; 35 -> 3.5 -> 4
        self.assertEqual(invoice_total([(1, 25)], 1000), 27)
        self.assertEqual(invoice_total([(1, 35)], 1000), 39)

    def test_discount(self):
        # subtotal 1005, discount 100.5 -> 101 (half-up), taxable 904
        self.assertEqual(invoice_total([(1, 1005)], 0, "TENOFF"), 904)

    def test_bad_inputs(self):
        with self.assertRaises(ValueError):
            invoice_total([(0, 100)], 0)
        with self.assertRaises(ValueError):
            invoice_total([(1, 100)], 0, "FREE")


class RenderTags(unittest.TestCase):
    def test_sorted(self):
        self.assertEqual(render_tags({"urgent", "net30"}), "net30, urgent")

    def test_empty(self):
        self.assertEqual(render_tags(set()), "")


if __name__ == "__main__":
    unittest.main()
