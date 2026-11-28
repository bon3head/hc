"""Slugs (SPEC.md section 1)."""

import re

_SPECIAL = {"ß": "ss", "æ": "ae", "ø": "o", "œ": "oe", "ł": "l"}


def slugify(text: str, sep: str = "-") -> str:
    text = text.lower()
    for k, v in _SPECIAL.items():
        text = text.replace(k, v)
    text = re.sub(r"[^a-z0-9]", sep, text)
    return text.strip(sep)
