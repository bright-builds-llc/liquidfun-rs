//! Concurrent bounded child stdout and stderr drains.

use std::{
    collections::VecDeque,
    io::{self, Read},
    process::{ChildStderr, ChildStdout},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use liquidfun_test_protocol::HarnessFailureKind;

const READ_CHUNK_BYTES: usize = 16 * 1024;
const SANITIZER_SCAN_OVERLAP: usize = 128;

#[derive(Debug)]
pub(super) enum IoEvent {
    StdoutRecord(Vec<u8>),
    StdoutPartial,
    StdoutEof,
    StdoutRecordTooLarge,
    OutputProgress(usize),
    SanitizerDetected,
    ReadFailure,
}

#[derive(Debug, Default)]
pub(super) struct StderrSnapshot {
    pub(super) retained: Vec<u8>,
    pub(super) total_bytes: usize,
}

pub(super) struct IoWorkers {
    receiver: Receiver<IoEvent>,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<StderrSnapshot>>,
    total_output: Arc<AtomicUsize>,
    sanitizer_detected: Arc<AtomicBool>,
}

impl IoWorkers {
    pub(super) fn spawn(
        stdout: ChildStdout,
        stderr: ChildStderr,
        output_record_bytes: usize,
        retained_stderr_bytes: usize,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let total_output = Arc::new(AtomicUsize::new(0));
        let sanitizer_detected = Arc::new(AtomicBool::new(false));
        let stdout_handle = {
            let sender = sender.clone();
            let total_output = Arc::clone(&total_output);
            thread::spawn(move || {
                drain_stdout(stdout, output_record_bytes, &sender, &total_output);
            })
        };
        let stderr_handle = {
            let total_output = Arc::clone(&total_output);
            let sanitizer_detected = Arc::clone(&sanitizer_detected);
            thread::spawn(move || {
                drain_stderr(
                    stderr,
                    retained_stderr_bytes,
                    &sender,
                    &total_output,
                    &sanitizer_detected,
                )
            })
        };

        Self {
            receiver,
            stdout_handle: Some(stdout_handle),
            stderr_handle: Some(stderr_handle),
            total_output,
            sanitizer_detected,
        }
    }

    pub(super) fn total_output(&self) -> usize {
        self.total_output.load(Ordering::Acquire)
    }

    pub(super) fn sanitizer_detected(&self) -> bool {
        self.sanitizer_detected.load(Ordering::Acquire)
    }

    pub(super) fn receive_until(
        &self,
        deadline: Instant,
        timeout_kind: HarnessFailureKind,
    ) -> Result<IoEvent, HarnessFailureKind> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(timeout_kind);
        }
        self.receiver
            .recv_timeout(remaining)
            .map_err(|error| match error {
                RecvTimeoutError::Timeout => timeout_kind,
                RecvTimeoutError::Disconnected => HarnessFailureKind::UnexpectedEof,
            })
    }

    pub(super) fn receive_optional_until(
        &self,
        deadline: Instant,
    ) -> Result<Option<IoEvent>, HarnessFailureKind> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(None);
        }
        match self.receiver.recv_timeout(remaining) {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(HarnessFailureKind::UnexpectedEof),
        }
    }

    pub(super) fn join(&mut self) -> StderrSnapshot {
        if let Some(stdout_handle) = self.stdout_handle.take() {
            let _stdout_joined = stdout_handle.join();
        }
        let Some(stderr_handle) = self.stderr_handle.take() else {
            return StderrSnapshot::default();
        };
        stderr_handle.join().unwrap_or_default()
    }
}

fn drain_stdout(
    mut stdout: ChildStdout,
    output_record_bytes: usize,
    sender: &Sender<IoEvent>,
    total_output: &AtomicUsize,
) {
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    let mut record = Vec::new();
    let mut record_too_large = false;
    loop {
        let count = match stdout.read(&mut chunk) {
            Ok(0) => {
                if !record.is_empty() || record_too_large {
                    let _partial_sent = sender.send(IoEvent::StdoutPartial);
                }
                let _eof_sent = sender.send(IoEvent::StdoutEof);
                return;
            }
            Ok(count) => count,
            Err(_) => {
                let _failure_sent = sender.send(IoEvent::ReadFailure);
                return;
            }
        };
        publish_progress(count, sender, total_output);
        for byte in &chunk[..count] {
            if record_too_large {
                if *byte == b'\n' {
                    record_too_large = false;
                    record.clear();
                }
                continue;
            }
            record.push(*byte);
            if record.len() > output_record_bytes {
                record_too_large = true;
                let _oversized_sent = sender.send(IoEvent::StdoutRecordTooLarge);
                continue;
            }
            if *byte == b'\n' {
                let complete = std::mem::take(&mut record);
                if sender.send(IoEvent::StdoutRecord(complete)).is_err() {
                    return;
                }
            }
        }
    }
}

fn drain_stderr(
    mut stderr: ChildStderr,
    retained_stderr_bytes: usize,
    sender: &Sender<IoEvent>,
    total_output: &AtomicUsize,
    sanitizer_detected: &AtomicBool,
) -> StderrSnapshot {
    let first_capacity = retained_stderr_bytes / 2;
    let tail_capacity = retained_stderr_bytes - first_capacity;
    let mut first = Vec::with_capacity(first_capacity);
    let mut tail = VecDeque::with_capacity(tail_capacity);
    let mut total_bytes = 0_usize;
    let mut scan_overlap = Vec::with_capacity(SANITIZER_SCAN_OVERLAP);
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    loop {
        let count = match stderr.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => count,
            Err(_) => {
                let _failure_sent = sender.send(IoEvent::ReadFailure);
                break;
            }
        };
        total_bytes = total_bytes.saturating_add(count);
        publish_progress(count, sender, total_output);
        retain_first_and_last(
            &chunk[..count],
            first_capacity,
            tail_capacity,
            &mut first,
            &mut tail,
        );
        if !sanitizer_detected.load(Ordering::Acquire)
            && contains_sanitizer_marker(&scan_overlap, &chunk[..count])
        {
            sanitizer_detected.store(true, Ordering::Release);
            let _sanitizer_sent = sender.send(IoEvent::SanitizerDetected);
        }
        update_scan_overlap(&mut scan_overlap, &chunk[..count]);
    }

    let mut retained = first;
    retained.extend(tail);
    StderrSnapshot {
        retained,
        total_bytes,
    }
}

fn publish_progress(count: usize, sender: &Sender<IoEvent>, total_output: &AtomicUsize) {
    let total = total_output
        .fetch_add(count, Ordering::AcqRel)
        .saturating_add(count);
    let _progress_sent = sender.send(IoEvent::OutputProgress(total));
}

fn retain_first_and_last(
    bytes: &[u8],
    first_capacity: usize,
    tail_capacity: usize,
    first: &mut Vec<u8>,
    tail: &mut VecDeque<u8>,
) {
    for byte in bytes {
        if first.len() < first_capacity {
            first.push(*byte);
            continue;
        }
        if tail_capacity == 0 {
            continue;
        }
        if tail.len() == tail_capacity {
            tail.pop_front();
        }
        tail.push_back(*byte);
    }
}

fn contains_sanitizer_marker(overlap: &[u8], bytes: &[u8]) -> bool {
    const MARKERS: [&[u8]; 4] = [
        b"ERROR: AddressSanitizer",
        b"SUMMARY: AddressSanitizer",
        b"UndefinedBehaviorSanitizer",
        b"runtime error:",
    ];
    let mut combined = Vec::with_capacity(overlap.len() + bytes.len());
    combined.extend_from_slice(overlap);
    combined.extend_from_slice(bytes);
    MARKERS.iter().any(|marker| {
        combined
            .windows(marker.len())
            .any(|window| window == *marker)
    })
}

fn update_scan_overlap(overlap: &mut Vec<u8>, bytes: &[u8]) {
    let keep_from_bytes = bytes.len().min(SANITIZER_SCAN_OVERLAP);
    if keep_from_bytes == SANITIZER_SCAN_OVERLAP {
        overlap.clear();
        overlap.extend_from_slice(&bytes[bytes.len() - keep_from_bytes..]);
        return;
    }
    let keep_from_overlap = (SANITIZER_SCAN_OVERLAP - keep_from_bytes).min(overlap.len());
    let overlap_start = overlap.len() - keep_from_overlap;
    let mut next = Vec::with_capacity(SANITIZER_SCAN_OVERLAP);
    next.extend_from_slice(&overlap[overlap_start..]);
    next.extend_from_slice(bytes);
    *overlap = next;
}

#[allow(
    dead_code,
    reason = "documents that drain failures are intentionally converted to events"
)]
fn _io_error_is_send_sync(_: io::Error) {}
