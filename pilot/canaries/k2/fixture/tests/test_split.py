from allot.model import Share
from allot.split import split_even, split_weighted


def raises(fn, *args):
    try:
        fn(*args)
    except ValueError:
        return True
    return False


def test_even_split_exact():
    assert split_even(90, ["a", "b", "c"]) == [Share("a", 30), Share("b", 30), Share("c", 30)]


def test_remainder_goes_to_first():
    assert split_even(100, ["a", "b", "c"]) == [Share("a", 34), Share("b", 33), Share("c", 33)]


def test_two_cents_left_over():
    assert split_even(11, ["x", "y", "z"]) == [Share("x", 4), Share("y", 4), Share("z", 3)]


def test_order_is_kept():
    assert [s.name for s in split_even(10, ["z", "a"])] == ["z", "a"]


def test_single_name_gets_everything():
    assert split_even(7, ["solo"]) == [Share("solo", 7)]


def test_zero_total():
    assert split_even(0, ["a", "b"]) == [Share("a", 0), Share("b", 0)]


def test_even_rejects_bad_input():
    assert raises(split_even, 10, [])
    assert raises(split_even, 10, ["a", "a"])
    assert raises(split_even, -1, ["a"])


def test_weighted_exact():
    assert split_weighted(100, {"a": 1, "b": 3}) == [Share("a", 25), Share("b", 75)]


def test_weighted_largest_remainder():
    # 100 * 1/3 = 33.33, 100 * 2/3 = 66.67 -> b has the larger remainder
    assert split_weighted(100, {"a": 1, "b": 2}) == [Share("a", 33), Share("b", 67)]


def test_weighted_tie_goes_to_earlier_name():
    assert split_weighted(10, {"c": 1, "a": 1, "b": 1}) == [Share("a", 4), Share("b", 3), Share("c", 3)]


def test_weighted_sums_to_total():
    shares = split_weighted(1001, {"p": 7, "q": 5, "r": 3})
    assert sum(s.cents for s in shares) == 1001


def test_weighted_rejects_bad_weights():
    assert raises(split_weighted, 10, {})
    assert raises(split_weighted, 10, {"a": 0})
