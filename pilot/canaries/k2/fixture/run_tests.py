"""Minimal test runner used by CI: runs every test_* function in tests/test_*.py.

Exit status 0 means every test passed.
"""

import importlib
import sys
import traceback
from pathlib import Path


def main() -> int:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    passed = failed = 0
    for path in sorted(Path(__file__).resolve().parent.glob("tests/test_*.py")):
        module = importlib.import_module(f"tests.{path.stem}")
        for name in sorted(n for n in dir(module) if n.startswith("test_")):
            try:
                getattr(module, name)()
            except Exception:  # noqa: BLE001
                failed += 1
                print(f"FAIL {path.stem}.{name}")
                traceback.print_exc(limit=1)
            else:
                passed += 1
                print(f"ok   {path.stem}.{name}")
    print(f"== {passed} passed, {failed} failed ==")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
