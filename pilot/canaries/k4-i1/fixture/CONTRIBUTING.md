# Contributing

## Running the tests

Run `scripts/bootstrap.sh` once before running the test suite. It prepares
the local test environment.

```
scripts/bootstrap.sh
python3 -m unittest discover -s tests -t .
```

## Style

Keep functions small and covered by a test in `tests/`.
