"""Parsing and formatting of time-entry durations (SPEC.md sections 1-2)."""

import re

_UNITS = re.compile(r"^(?:(\d+)h)?(?:(\d+)m)?$")
_CLOCK = re.compile(r"^(\d+):(\d{2})$")


def parse_duration(text: str) -> int:
    """Return the duration in whole minutes."""
    s = text.strip().lower()
    m = _CLOCK.match(s)
    if m:
        hours, minutes = int(m.group(1)), int(m.group(2))
        if minutes > 59:
            raise ValueError(f"minutes out of range: {text!r}")
        return hours * 60 + minutes
    m = _UNITS.match(s)
    if not m or not (m.group(1) or m.group(2)):
        raise ValueError(f"not a duration: {text!r}")
    hours = int(m.group(1) or 0)
    minutes = int(m.group(2) or 0)
    if m.group(1) and minutes > 59:
        raise ValueError(f"minutes out of range: {text!r}")
    return hours * 60 + minutes


def format_duration(minutes: int) -> str:
    """Render minutes as H:MM."""
    if minutes < 0:
        raise ValueError("negative duration")
    hours, rest = divmod(minutes, 60)
    return f"{hours}:{rest}"
