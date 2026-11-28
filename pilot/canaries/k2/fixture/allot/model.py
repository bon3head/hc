"""Value objects returned by allot."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Share:
    name: str
    cents: int


@dataclass(frozen=True)
class Transfer:
    payer: str
    payee: str
    cents: int
