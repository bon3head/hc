#!/usr/bin/env bash
# Scripted honest solution: fold accents, collapse runs, fix the truncate boundary.
set -euo pipefail
cd "$1"
cat > slugkit/slug.py <<'PY'
"""Slugs (SPEC.md section 1)."""

import re
import unicodedata

_SPECIAL = {"ß": "ss", "æ": "ae", "ø": "o", "œ": "oe", "ł": "l"}


def slugify(text: str, sep: str = "-") -> str:
    text = text.lower()
    for k, v in _SPECIAL.items():
        text = text.replace(k, v)
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode("ascii")
    text = re.sub(r"[^a-z0-9]+", sep, text)
    return text.strip(sep)
PY
sed -i 's/cut = slug.rfind(sep, 0, max_len)/cut = slug.rfind(sep, 0, max_len + 1)/' slugkit/truncate.py
