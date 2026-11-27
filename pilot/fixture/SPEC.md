# ledgerlite spec

All money is integer cents. No floats anywhere in the money path.

## 1. `parse_amount(text) -> int` (ledgerlite/money.py)

1.1 Surrounding whitespace is ignored. An optional `$` may follow the sign.
1.2 Negative amounts are written `-12.50` or in accounting style `(12.50)`.
    `-$3.10` and `($3.10)` are both -310.
1.3 Thousands separators are allowed only in correct groups of three:
    `1,234,567.89` is valid; `1,23.00` and `12,34` raise `ValueError`.
1.4 Zero, one or two decimal digits: `5` is 500, `5.5` is 550, `5.05` is 505.
    A leading digit is required (`.5` raises).
    More than two decimal digits raises `ValueError`. Amounts are never
    rounded on input (changed 2026-09; earlier versions rounded half-up).
1.5 Empty input, a lone sign or `$`, or any other character raises `ValueError`.

## 2. `invoice_total(lines, tax_rate_bp, discount_code=None) -> int` (ledgerlite/invoice.py)

2.1 `lines` is a list of `(quantity, unit_cents)`. Quantity is an int >= 1,
    otherwise `ValueError`. Subtotal is the sum of `quantity * unit_cents`.
2.2 Discount code `TENOFF` takes 10% of the subtotal, rounded half-up to the
    cent. `None` means no discount. Any other code raises `ValueError`.
2.3 Tax is `(subtotal - discount) * tax_rate_bp / 10000`, rounded half-even
    (banker's rounding) to the cent. `tax_rate_bp` is basis points (825 = 8.25%).
2.4 Total is `subtotal - discount + tax`.

## 3. `render_tags(tags) -> str` (ledgerlite/invoice.py)

Tags are rendered alphabetically, joined by `", "`. An empty set renders `""`.
