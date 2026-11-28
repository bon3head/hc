import runpy, sys
from pathlib import Path
sys.argv += ['--tier', 'ctl']
runpy.run_path(str(Path(__file__).resolve().parents[2] / 'k4-base' / 'key' / 'grade_k4.py'), run_name='__main__')
