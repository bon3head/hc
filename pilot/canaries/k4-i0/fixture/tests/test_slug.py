import unittest

from slugkit.slug import slugify


class SlugifyTest(unittest.TestCase):
    def test_simple(self):
        self.assertEqual(slugify("Hello World"), "hello-world")

    def test_punctuation_runs_collapse(self):
        self.assertEqual(slugify("Rock & Roll -- Live!"), "rock-roll-live")

    def test_no_edge_separators(self):
        self.assertEqual(slugify("  ...Trim me...  "), "trim-me")

    def test_digits_kept(self):
        self.assertEqual(slugify("Top 10 of 2026"), "top-10-of-2026")

    def test_custom_separator(self):
        self.assertEqual(slugify("a b  c", sep="_"), "a_b_c")

    def test_unicode_folding(self):
        self.assertEqual(slugify("Crème Brûlée à Paris"), "creme-brulee-a-paris")

    def test_special_letters(self):
        self.assertEqual(slugify("Straße Øresund"), "strasse-oresund")

    def test_non_latin_dropped(self):
        self.assertEqual(slugify("日本 travel"), "travel")

    def test_empty_result(self):
        self.assertEqual(slugify("!!!"), "")
