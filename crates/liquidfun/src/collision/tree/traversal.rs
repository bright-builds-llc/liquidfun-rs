//! Borrow-scoped tree traversal is implemented by Plan 05-05 Task 2.

/// Continue or stop an AABB query visitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryControl {
    /// Continue visiting candidate leaves.
    Continue,
    /// Stop the query immediately.
    Stop,
}

/// Ignore, terminate, or clip a ray traversal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayCastControl {
    /// Ignore this candidate and preserve the current ray interval.
    Ignore,
    /// Terminate traversal immediately.
    Terminate,
    /// Clip subsequent traversal to the supplied normalized fraction.
    Clip(f32),
}
