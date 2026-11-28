"""Billing increments (SPEC.md section 3)."""


def round_up(minutes: int, increment: int = 6) -> int:
    """Round a time entry up to the billing increment."""
    if increment < 1:
        raise ValueError("increment must be at least 1")
    if minutes < 0:
        raise ValueError("negative minutes")
    return minutes // increment * increment
