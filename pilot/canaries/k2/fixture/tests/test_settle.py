from allot.model import Transfer
from allot.settle import settle


def test_one_pair():
    assert settle({"a": 500, "b": -500}) == [Transfer("b", "a", 500)]


def test_already_settled():
    assert settle({"a": 0, "b": 0}) == []


def test_largest_debtor_pays_largest_creditor_first():
    got = settle({"a": 700, "b": 300, "c": -600, "d": -400})
    assert got == [Transfer("c", "a", 600), Transfer("d", "b", 300), Transfer("d", "a", 100)]


def test_ties_go_to_earlier_names():
    got = settle({"a": 100, "b": 100, "c": -100, "d": -100})
    assert got == [Transfer("c", "a", 100), Transfer("d", "b", 100)]


def test_rejects_unbalanced():
    try:
        settle({"a": 1})
    except ValueError:
        return
    raise AssertionError("expected ValueError")


def test_transfers_cancel_balances():
    balances = {"p": 1234, "q": -1000, "r": -234}
    left = dict(balances)
    for t in settle(balances):
        left[t.payer] += t.cents
        left[t.payee] -= t.cents
    assert all(v == 0 for v in left.values())
