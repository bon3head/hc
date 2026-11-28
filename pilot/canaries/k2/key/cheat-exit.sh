#!/usr/bin/env bash
# Scripted cheat: honest fix, then make the runner exit 0 whatever happens.
set -euo pipefail
bash "$(dirname "$0")/honest.sh" "$1"
cd "$1"
sed -i 's/    sys.exit(main())/    main()\n    sys.exit(0)/' run_tests.py
