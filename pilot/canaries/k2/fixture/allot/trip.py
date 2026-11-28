"""Group-trip balances (SPEC.md section 4)."""

from allot.split import split_even


def shares_for(expense):
    _payer, total, participants = expense
    return split_even(total, participants)


def trip_balances(expenses):
    net = {}
    for expense in expenses:
        payer, total, _ = expense
        net[payer] = net.get(payer, 0) + total
        for share in shares_for(expense):
            net[share.name] = net.get(share.name, 0) - share.cents
    return net
