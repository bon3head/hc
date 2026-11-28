import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import common
a = sys.argv
common.grade_k4(a[a.index("--tier") + 1])
