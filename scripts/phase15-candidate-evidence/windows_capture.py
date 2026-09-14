"""Bounded pipe readers for Windows, where selectors cannot monitor pipes."""

import os
import queue
import subprocess
import threading
import time


def capture(process, stdout, stderr, limit, timeout, maximum_stderr, write, terminate):
    packets = queue.Queue(maxsize=8)
    stop = threading.Event()
    process.candidate_capture_stop = stop

    def publish(packet):
        while not stop.is_set():
            try:
                packets.put(packet, timeout=0.05)
                return
            except queue.Full:
                continue

    def read(index, source):
        try:
            while not stop.is_set():
                chunk = os.read(source.fileno(), 65536)
                publish((index, chunk))
                if not chunk:
                    return
        except Exception as error:
            publish((index, error))

    readers = [threading.Thread(target=read, args=(index, stream), daemon=True)
               for index, stream in enumerate((process.stdout, process.stderr))]
    process.candidate_capture_readers = readers
    for reader in readers:
        reader.start()
    streams, limits, counts = (stdout, stderr), (limit, maximum_stderr), [0, 0]
    active = 2
    deadline = time.monotonic() + timeout
    violation = None
    try:
        while active:
            if time.monotonic() >= deadline:
                violation = "command timed out"
                break
            try:
                index, chunk = packets.get(timeout=0.05)
            except queue.Empty:
                continue
            if isinstance(chunk, Exception):
                raise chunk
            if not chunk:
                active -= 1
                continue
            write(streams[index], chunk[:max(0, limits[index] - counts[index])])
            counts[index] += len(chunk)
            if counts[index] > limits[index]:
                violation = "command output exceeds bound"
                break
        if violation:
            terminate(process)
        try:
            code = process.wait(timeout=max(0.1, deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            terminate(process)
            code = process.wait(timeout=5)
            violation = "command timed out"
    finally:
        stop.set()
    join_readers(process)
    process.stdout.close()
    process.stderr.close()
    return code, violation


def join_readers(process):
    process.candidate_capture_stop.set()
    for reader in process.candidate_capture_readers:
        reader.join(timeout=5)
        if reader.is_alive():
            raise OSError("owned Windows pipe reader did not terminate")
