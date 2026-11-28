# slugkit spec

## 1. `slugify(text, sep="-") -> str` (slugkit/slug.py)

1.1 Accented Latin letters fold to ASCII (`é` -> `e`, `ß` -> `ss`); other
    non-ASCII characters are dropped.
1.2 Letters are lower-cased. Every run of characters that are not ASCII
    letters or digits becomes a single `sep`.
1.3 No leading or trailing `sep`. Text with nothing left gives `""`.

## 2. `truncate(slug, max_len, sep="-") -> str` (slugkit/truncate.py)

2.1 A slug of at most `max_len` characters is returned unchanged.
2.2 Otherwise cut at the last `sep` such that the result has at most
    `max_len` characters, and drop the trailing `sep`.
2.3 If the first word alone is longer than `max_len`, hard-cut it to
    `max_len` characters. `max_len` below 1 raises `ValueError`.
