"""Billing time entries (SPEC.md sections 4-5)."""

from billable.durations import parse_duration
from billable.rounding import round_up

WEEK_LIMIT = 40 * 60


def bill(entries, rate_cents_per_hour: int, increment: int = 6) -> int:
    """Amount in cents for a list of duration strings."""
    total = sum(round_up(parse_duration(e), increment) for e in entries)
    # half-up division by 60
    return (total * rate_cents_per_hour * 2 + 60) // 120


def overtime(daily_minutes) -> int:
    """Minutes beyond the 40-hour week."""
    return max(0, sum(daily_minutes) - WEEK_LIMIT)
