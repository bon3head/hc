"""Retail 4-4-5 fiscal calendar (SPEC.md)."""

from datetime import date, timedelta

PATTERN = (4, 4, 5) * 4


def _nearest_saturday(d: date) -> date:
    # step back to the Saturday on or before d
    return d - timedelta(days=(d.weekday() - 5) % 7)


def fiscal_year_end(fy: int) -> date:
    """Last day of fiscal year `fy`: the Saturday nearest January 31 of fy + 1."""
    return _nearest_saturday(date(fy + 1, 1, 31))


def fiscal_year_start(fy: int) -> date:
    return fiscal_year_end(fy - 1) + timedelta(days=1)


def weeks_in_year(fy: int) -> int:
    return ((fiscal_year_end(fy) - fiscal_year_start(fy)).days + 1) // 7


def _period_lengths(fy: int):
    lengths = list(PATTERN)
    if weeks_in_year(fy) == 53:
        lengths[0] += 1
    return lengths


def fiscal_week(d: date):
    """(fiscal year, week 1-53) containing `d`."""
    fy = d.year - 1 if d <= fiscal_year_end(d.year - 1) else d.year
    return fy, (d - fiscal_year_start(fy)).days // 7 + 1


def fiscal_period(d: date):
    """(fiscal year, period 1-12) containing `d`."""
    fy, week = fiscal_week(d)
    for period, length in enumerate(_period_lengths(fy), start=1):
        if week <= length:
            return fy, period
        week -= length
    raise AssertionError("unreachable")


def period_bounds(fy: int, period: int):
    """(first day, last day) of a fiscal period."""
    if not 1 <= period <= 12:
        raise ValueError("period must be 1-12")
    lengths = _period_lengths(fy)
    start = fiscal_year_start(fy) + timedelta(weeks=sum(lengths[: period - 1]))
    return start, start + timedelta(weeks=lengths[period - 1], days=-1)
