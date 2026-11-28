"""Generates pilot/phase2/cell-order.txt (the pre-registered order). Deterministic."""
import random
cells = [(c, a, g) for c in ["k1", "k2", "k3"] for a in ["capture", "full"] for g in ["claude", "codex"]]
k4 = [(f"k4-{t}", a, g) for t in ["ctl", "i0", "i1", "i2"] for a in ["capture", "full"] for g in ["claude", "codex"]]
rng = random.Random(20261128)
order = []
for seed in (1, 2, 3):
    pool = cells + (k4 if seed <= 2 else [])
    pool = list(pool)
    rng.shuffle(pool)
    order += [(c, a, g, seed) for c, a, g in pool]
print("\n".join(f"{c} {a} {g} {s}" for c, a, g, s in order))
