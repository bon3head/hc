# allot spec

Money is integer cents. Every function returns value objects from
`allot/model.py`; callers compare them with `==`.

## 1. `split_even(total, names) -> list[Share]` (allot/split.py)

1.1 Splits `total` cents across `names` as evenly as possible, one `Share`
    per name, in the order given.
1.2 Leftover cents go one each to the **first** names in the order given:
    100 across a, b, c is 34, 33, 33.
1.3 `names` must be non-empty and unique and `total` >= 0; otherwise `ValueError`.

## 2. `split_weighted(total, weights) -> list[Share]` (allot/split.py)

2.1 `weights` maps name -> positive integer weight. Shares are returned in
    name order (sorted).
2.2 Each name first gets `floor(total * w / sum(w))`. Leftover cents go one
    each to the names with the largest fractional remainders; equal
    remainders go to names earlier in sorted order.
2.3 An empty mapping or a weight below 1 raises `ValueError`.

## 3. `settle(balances) -> list[Transfer]` (allot/settle.py)

3.1 `balances` maps name -> net cents (positive: is owed, negative: owes).
    They must sum to zero, otherwise `ValueError`.
3.2 Repeatedly the largest debtor pays the largest creditor as much as
    possible (ties: earlier name in sorted order) until all are zero.
3.3 Returns transfers in the order they were made; zero balances produce none.

## 4. Trips (allot/trip.py)

4.1 `shares_for(expense) -> list[Share]`: an expense is `(payer, total,
    participants)` and is split with `split_even` across its participants in
    the order listed.
4.2 `trip_balances(expenses) -> dict`: `expenses` is a list of expenses, each split per 4.1; the payer is
credited `total` and each participant is debited their share. Returns
name -> net cents for everyone who appears.
