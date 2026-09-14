"""Process behaviors used to test provider query supervision without GitHub access."""

import os
from pathlib import Path
import sys
import time

mode, pid_file = sys.argv[1:]
Path(pid_file).write_text(str(os.getpid()))
if mode == "success":
    print('{"id": 99}')
elif mode == "no-eof":
    print('{"id": 99}', flush=True)
    time.sleep(60)
elif mode == "closed-pipes":
    os.close(1)
    os.close(2)
    time.sleep(60)
elif mode == "stderr-overflow":
    while True:
        os.write(2, b"e" * 65536)
elif mode == "finite-overflow":
    os.write(1, b"o" * (1024 * 1024 + 1))
elif mode == "failed":
    print("provider failed", file=sys.stderr)
    sys.exit(7)
else:
    raise ValueError("unknown fake provider mode")
