//! Concurrent bounded child stdout and stderr drains.

use std::{
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

struct StderrRetention {
    first: Vec<u8>,
    tail: Vec<u8>,
    first_capacity: usize,
    tail_capacity: usize,
    tail_start: usize,
}

impl StderrRetention {
    fn new(capacity: usize) -> Self {
        let first_capacity = capacity / 2;
        let tail_capacity = capacity - first_capacity;
        Self {
            first: Vec::with_capacity(first_capacity),
            tail: Vec::with_capacity(tail_capacity),
            first_capacity,
            tail_capacity,
            tail_start: 0,
        }
    }

    fn retain(&mut self, bytes: &[u8]) {
        let first_count = bytes
            .len()
            .min(self.first_capacity.saturating_sub(self.first.len()));
        self.first.extend_from_slice(&bytes[..first_count]);
        let tail_bytes = &bytes[first_count..];
        if tail_bytes.is_empty() || self.tail_capacity == 0 {
            return;
        }
        if tail_bytes.len() >= self.tail_capacity {
            self.tail.clear();
            self.tail
                .extend_from_slice(&tail_bytes[tail_bytes.len() - self.tail_capacity..]);
            self.tail_start = 0;
            return;
        }

        let fill_count = tail_bytes
            .len()
            .min(self.tail_capacity.saturating_sub(self.tail.len()));
        self.tail.extend_from_slice(&tail_bytes[..fill_count]);
        let wrapped = &tail_bytes[fill_count..];
        if wrapped.is_empty() {
            return;
        }

        let first_write = wrapped.len().min(self.tail_capacity - self.tail_start);
        self.tail[self.tail_start..self.tail_start + first_write]
            .copy_from_slice(&wrapped[..first_write]);
        self.tail[..wrapped.len() - first_write].copy_from_slice(&wrapped[first_write..]);
        self.tail_start = (self.tail_start + wrapped.len()) % self.tail_capacity;
    }

    fn into_retained(self) -> Vec<u8> {
        let mut retained = self.first;
        retained.extend_from_slice(&self.tail[self.tail_start..]);
        retained.extend_from_slice(&self.tail[..self.tail_start]);
        retained
    }
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
    let mut retention = StderrRetention::new(retained_stderr_bytes);
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
        retention.retain(&chunk[..count]);
        if !sanitizer_detected.load(Ordering::Acquire)
            && contains_sanitizer_marker(&scan_overlap, &chunk[..count])
        {
            sanitizer_detected.store(true, Ordering::Release);
            let _sanitizer_sent = sender.send(IoEvent::SanitizerDetected);
        }
        update_scan_overlap(&mut scan_overlap, &chunk[..count]);
    }

    StderrSnapshot {
        retained: retention.into_retained(),
        total_bytes,
    }
}

fn publish_progress(count: usize, sender: &Sender<IoEvent>, total_output: &AtomicUsize) {
    let total = total_output
        .fetch_add(count, Ordering::AcqRel)
        .saturating_add(count);
    let _progress_sent = sender.send(IoEvent::OutputProgress(total));
}

fn contains_sanitizer_marker(overlap: &[u8], bytes: &[u8]) -> bool {
    let mut combined = Vec::with_capacity(overlap.len() + bytes.len());
    combined.extend_from_slice(overlap);
    combined.extend_from_slice(bytes);
    combined.iter().enumerate().any(|(index, byte)| {
        let remaining = &combined[index..];
        match *byte {
            b'E' => remaining.starts_with(b"ERROR: AddressSanitizer"),
            b'S' => remaining.starts_with(b"SUMMARY: AddressSanitizer"),
            b'U' => remaining.starts_with(b"UndefinedBehaviorSanitizer"),
            b'r' => remaining.starts_with(b"runtime error:"),
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::{StderrRetention, contains_sanitizer_marker};

    #[test]
    fn chunk_retention_preserves_first_and_last_across_wraps() {
        // Arrange
        let mut retention = StderrRetention::new(10);

        // Act
        retention.retain(b"abc");
        retention.retain(b"defgh");
        retention.retain(b"ijkl");

        // Assert
        assert_eq!(retention.into_retained(), b"abcdehijkl");
    }

    #[test]
    fn chunk_retention_keeps_latest_tail_from_an_oversized_chunk() {
        // Arrange
        let mut retention = StderrRetention::new(8);

        // Act
        retention.retain(b"abcdefghijkl");

        // Assert
        assert_eq!(retention.into_retained(), b"abcdijkl");
    }

    #[test]
    fn sanitizer_marker_detection_spans_read_chunks() {
        // Arrange
        let overlap = b"diagnostic: ERROR: Address";
        let bytes = b"Sanitizer: injected";

        // Act
        let detected = contains_sanitizer_marker(overlap, bytes);

        // Assert
        assert!(detected);
    }
}
