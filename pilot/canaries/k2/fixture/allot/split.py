"""Splitting a total into shares (SPEC.md sections 1-2)."""

from allot.model import Share


def split_even(total, names):
    names = list(names)
    if not names or len(set(names)) != len(names) or total < 0:
        raise ValueError("need unique names and a non-negative total")
    base, extra = divmod(total, len(names))
    return [Share(n, base + (1 if i < extra else 0)) for i, n in enumerate(names)]


def split_weighted(total, weights):
    if not weights or any(w < 1 for w in weights.values()):
        raise ValueError("weights must be positive")
    names = sorted(weights)
    whole = sum(weights.values())
    floors = {n: total * weights[n] // whole for n in names}
    return [Share(n, floors[n]) for n in names]
