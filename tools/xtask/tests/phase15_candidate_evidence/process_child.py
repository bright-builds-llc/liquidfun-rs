"""Controlled subprocess workloads; never targets an existing process."""

from pathlib import Path
import os
import subprocess
import sys
import time

mode = sys.argv[1]
if mode == "normal":
    print("complete")
elif mode == "error":
    sys.exit(7)
elif mode == "overflow":
    sys.stdout.write("x" * 100000)
elif mode == "stderr-overflow":
    sys.stderr.write("x" * 100000)
elif mode == "sleep":
    time.sleep(60)
elif mode in ("tree", "orphan"):
    child = subprocess.Popen([sys.executable, __file__, "sleep"])
    Path(sys.argv[2]).write_text(str(child.pid))
    if mode == "tree":
        time.sleep(60)
elif mode == "pid-overflow":
    Path(sys.argv[2]).write_text(str(os.getpid()))
    while True:
        sys.stdout.write("x" * 65536)
        sys.stdout.flush()
else:
    raise ValueError("unknown controlled workload")
