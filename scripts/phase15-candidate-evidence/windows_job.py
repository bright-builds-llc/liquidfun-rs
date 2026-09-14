"""An owned Windows Job Object confines every descendant of one command."""

import ctypes
from ctypes import wintypes
import subprocess


class BasicLimits(ctypes.Structure):
    _fields_ = [("process_time", ctypes.c_int64), ("job_time", ctypes.c_int64),
                ("flags", wintypes.DWORD), ("minimum_working_set", ctypes.c_size_t),
                ("maximum_working_set", ctypes.c_size_t), ("active_processes", wintypes.DWORD),
                ("affinity", ctypes.c_size_t), ("priority", wintypes.DWORD),
                ("scheduling", wintypes.DWORD)]


class IoCounters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint64) for name in
                ("read_ops", "write_ops", "other_ops", "read_bytes", "write_bytes", "other_bytes")]


class ExtendedLimits(ctypes.Structure):
    _fields_ = [("basic", BasicLimits), ("io", IoCounters),
                ("process_memory", ctypes.c_size_t), ("job_memory", ctypes.c_size_t),
                ("peak_process_memory", ctypes.c_size_t), ("peak_job_memory", ctypes.c_size_t)]


class ThreadEntry(ctypes.Structure):
    _fields_ = [("size", wintypes.DWORD), ("usage", wintypes.DWORD),
                ("thread_id", wintypes.DWORD), ("owner_pid", wintypes.DWORD),
                ("base_priority", wintypes.LONG), ("delta_priority", wintypes.LONG),
                ("flags", wintypes.DWORD)]


class OwnedJob:
    def __init__(self):
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
        self.kernel.CreateJobObjectW.restype = wintypes.HANDLE
        self.kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
        self.kernel.SetInformationJobObject.restype = wintypes.BOOL
        self.kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
        self.kernel.AssignProcessToJobObject.restype = wintypes.BOOL
        self.kernel.TerminateJobObject.argtypes = [wintypes.HANDLE, wintypes.UINT]
        self.kernel.TerminateJobObject.restype = wintypes.BOOL
        self.kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        self.kernel.CloseHandle.restype = wintypes.BOOL
        self.handle = self.kernel.CreateJobObjectW(None, None)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        limits = ExtendedLimits()
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not self.kernel.SetInformationJobObject(self.handle, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
            error = ctypes.WinError(ctypes.get_last_error())
            self.close()
            raise error

    def assign_and_resume(self, process):
        handle = wintypes.HANDLE(int(process._handle))
        if not self.kernel.AssignProcessToJobObject(self.handle, handle):
            raise ctypes.WinError(ctypes.get_last_error())
        resume_primary_thread(self.kernel, process.pid)

    def terminate(self):
        if self.handle and not self.kernel.TerminateJobObject(self.handle, 1):
            raise ctypes.WinError(ctypes.get_last_error())

    def close(self):
        if self.handle:
            if not self.kernel.CloseHandle(self.handle):
                raise ctypes.WinError(ctypes.get_last_error())
            self.handle = None


def resume_primary_thread(kernel, pid):
    """Use documented Toolhelp/OpenThread/ResumeThread APIs on our suspended PID only."""
    kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    for name in ("Thread32First", "Thread32Next"):
        function = getattr(kernel, name)
        function.argtypes = [wintypes.HANDLE, ctypes.POINTER(ThreadEntry)]
        function.restype = wintypes.BOOL
    kernel.OpenThread.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenThread.restype = wintypes.HANDLE
    kernel.ResumeThread.argtypes = [wintypes.HANDLE]
    kernel.ResumeThread.restype = wintypes.DWORD
    kernel.GetProcessIdOfThread.argtypes = [wintypes.HANDLE]
    kernel.GetProcessIdOfThread.restype = wintypes.DWORD
    snapshot = kernel.CreateToolhelp32Snapshot(4, 0)  # TH32CS_SNAPTHREAD; read-only snapshot.
    if snapshot == wintypes.HANDLE(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    threads = []
    try:
        entry = ThreadEntry()
        entry.size = ctypes.sizeof(entry)
        available = kernel.Thread32First(snapshot, ctypes.byref(entry))
        while available:
            if entry.owner_pid == pid:
                threads.append(entry.thread_id)
            entry.size = ctypes.sizeof(entry)
            available = kernel.Thread32Next(snapshot, ctypes.byref(entry))
        if ctypes.get_last_error() != 18:  # ERROR_NO_MORE_FILES
            raise ctypes.WinError(ctypes.get_last_error())
    finally:
        if not kernel.CloseHandle(snapshot):
            raise ctypes.WinError(ctypes.get_last_error())
    if len(threads) != 1:
        raise OSError("suspended command must have exactly one primary thread")
    thread = kernel.OpenThread(0x0802, False, threads[0])  # Suspend/resume and limited identity query.
    if not thread:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if kernel.GetProcessIdOfThread(thread) != pid:
            raise OSError("primary thread owner changed before resume")
        previous = kernel.ResumeThread(thread)
        if previous == 0xFFFFFFFF:
            raise ctypes.WinError(ctypes.get_last_error())
        if previous != 1:
            raise OSError("unexpected primary-thread suspend count")
    finally:
        if not kernel.CloseHandle(thread):
            raise ctypes.WinError(ctypes.get_last_error())


def spawn(args, root):
    job = OwnedJob()
    process = None
    try:
        # Suspended creation prevents a child escaping before assignment to our job.
        process = subprocess.Popen(args, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   creationflags=0x00000004)  # CREATE_SUSPENDED
        job.assign_and_resume(process)
        process.candidate_job = job
        return process
    except BaseException as original:
        for cleanup in ([process.kill, lambda: process.wait(timeout=5),
                         process.stdout.close, process.stderr.close] if process is not None else []):
            try:
                cleanup()
            except Exception as error:
                original.add_note(f"Suspended process cleanup failed: {error}")
        try:
            job.close()
        except Exception as error:
            original.add_note(f"Job cleanup failed: {error}")
        raise
