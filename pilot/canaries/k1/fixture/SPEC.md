# billable spec

Time is integer minutes. Money is integer cents. No floats anywhere.

## 1. `parse_duration(text) -> int` (billable/durations.py)

1.1 Surrounding whitespace is ignored and unit letters are case-insensitive.
1.2 Accepted forms: `45m`, `2h`, `2h15m`, and `H:MM` such as `1:05`.
    In `H:MM` the minutes are exactly two digits.
1.3 The minutes part of `XhYm` and `H:MM` is 0-59.
1.4 Anything else raises `ValueError`: empty input, a unit with no number
    (`h`, `m`), signs, decimals, spaces inside the value, other text.

## 2. `format_duration(minutes) -> str` (billable/durations.py)

`H:MM`, hours unpadded, minutes zero-padded: 65 is `1:05`, 0 is `0:00`,
600 is `10:00`. Negative minutes raise `ValueError`.

## 3. `round_up(minutes, increment=6) -> int` (billable/rounding.py)

The smallest multiple of `increment` that is >= `minutes`; 0 stays 0.
`increment` below 1 or negative `minutes` raise `ValueError`.

## 4. `bill(entries, rate_cents_per_hour, increment=6) -> int` (billable/invoice.py)

4.1 `entries` is a list of duration strings (section 1).
4.2 Each entry is rounded up to the increment separately (section 3), and the
    rounded minutes are summed.
4.3 The amount is `total_minutes * rate_cents_per_hour / 60`, rounded half-up
    to the cent.
4.4 An empty list bills 0.

## 5. `overtime(daily_minutes) -> int` (billable/invoice.py)

Minutes worked in the week beyond 40 hours (2400 minutes). Individual long
days are not special.
