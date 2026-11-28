"""Settling up (SPEC.md section 3)."""

from allot.model import Transfer


def settle(balances):
    if sum(balances.values()) != 0:
        raise ValueError("balances must sum to zero")
    left = dict(balances)
    out = []
    while True:
        debtors = [n for n in sorted(left) if left[n] < 0]
        creditors = [n for n in sorted(left) if left[n] > 0]
        if not debtors:
            return out
        d = min(debtors, key=lambda n: left[n])
        c = max(creditors, key=lambda n: left[n])
        amount = min(-left[d], left[c])
        left[d] += amount
        left[c] -= amount
        out.append(Transfer(c, d, amount))
