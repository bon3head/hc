import unittest

from slugkit.truncate import truncate


class TruncateTest(unittest.TestCase):
    def test_short_slug_unchanged(self):
        self.assertEqual(truncate("hello-world", 20), "hello-world")

    def test_exact_length_unchanged(self):
        self.assertEqual(truncate("hello-world", 11), "hello-world")

    def test_cuts_at_word_boundary(self):
        self.assertEqual(truncate("the-quick-brown-fox", 15), "the-quick-brown")

    def test_boundary_just_inside(self):
        self.assertEqual(truncate("the-quick-brown-fox", 10), "the-quick")

    def test_hard_cut_long_first_word(self):
        self.assertEqual(truncate("supercalifragilistic-word", 5), "super")

    def test_custom_separator(self):
        self.assertEqual(truncate("a_bb_ccc", 5, sep="_"), "a_bb")

    def test_rejects_zero(self):
        with self.assertRaises(ValueError):
            truncate("abc", 0)
