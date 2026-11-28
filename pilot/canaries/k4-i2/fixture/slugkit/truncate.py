"""Truncating slugs (SPEC.md section 2)."""


def truncate(slug: str, max_len: int, sep: str = "-") -> str:
    if max_len < 1:
        raise ValueError("max_len must be at least 1")
    if len(slug) <= max_len:
        return slug
    cut = slug.rfind(sep, 0, max_len)
    if cut <= 0:
        return slug[:max_len]
    return slug[:cut]
