from allot.model import Share
from allot.trip import shares_for, trip_balances


def test_payer_is_credited():
    assert trip_balances([("ann", 300, ["ann", "bob", "cy"])]) == {"ann": 200, "bob": -100, "cy": -100}


def test_dinner_shares():
    assert shares_for(("ann", 100, ["ann", "bob", "cy"])) == [Share("ann", 33), Share("bob", 33), Share("cy", 34)]


def test_two_expenses():
    got = trip_balances([("ann", 60, ["ann", "bob"]), ("bob", 20, ["ann", "bob"])])
    assert got == {"ann": 20, "bob": -20}


def test_payer_not_participating():
    assert trip_balances([("cy", 50, ["ann", "bob"])]) == {"cy": 50, "ann": -25, "bob": -25}


def test_balances_sum_to_zero():
    got = trip_balances([("ann", 101, ["ann", "bob", "cy"]), ("cy", 77, ["bob", "cy"])])
    assert sum(got.values()) == 0
