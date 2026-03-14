// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
use std::collections::VecDeque;

/// Default raw replay buffer capacity in bytes (512 KB).
///
/// When the replay buffer reaches its capacity, the oldest bytes are discarded
/// to make room for new output. This ensures the most recent terminal state is
/// always available for replay to xterm.js.
///
/// Orchestration terminals may override this to a larger value (e.g., 2 MB) via
/// `TerminalConfig::raw_replay_capacity_bytes` to avoid losing stream-json data
/// from large AI provider responses.
pub const DEFAULT_RAW_REPLAY_CAPACITY_BYTES: usize = 512 * 1024;

/// Ring buffer for terminal output with configurable capacity.
///
/// Stores clean text lines for detection/search, plus a raw byte replay buffer
/// (with ANSI sequences intact) for replaying the terminal to xterm.js.
///
/// **Incomplete line handling**: PTY output arrives in arbitrary byte chunks that
/// may split mid-line (no trailing newline). The buffer maintains a `pending` field
/// that accumulates text until a newline arrives. Only complete lines are committed
/// to the ring buffer. Call `flush()` to force the pending partial line into the
/// buffer (e.g., on EOF or when you need to read the current prompt line).
pub struct OutputBuffer {
    /// Completed lines (newline-terminated text that has been committed).
    lines: VecDeque<String>,
    /// Maximum number of lines to retain.
    capacity: usize,
    /// Partial line being accumulated (text received without a trailing newline).
    pending: String,
    /// Raw bytes (ANSI sequences intact) for replaying the terminal to xterm.js.
    /// Capped at `raw_capacity`; oldest bytes are dropped when full.
    raw_replay: Vec<u8>,
    /// Configurable capacity for the raw replay ring buffer (bytes).
    raw_capacity: usize,
    /// Cumulative count of all raw bytes ever pushed (not capped by ring buffer).
    /// Used by the idle detector to track whether new output is arriving, even
    /// when the ring buffer is full and `raw_len()` has plateaued.
    total_raw_bytes: usize,
}

impl OutputBuffer {
    /// Create a new output buffer with the given line capacity and raw replay
    /// byte capacity.
    ///
    /// `raw_capacity` controls the maximum size of the raw byte ring buffer
    /// used for xterm.js replay. Pass `DEFAULT_RAW_REPLAY_CAPACITY_BYTES`
    /// (512 KB) for normal terminals, or a larger value for orchestration
    /// terminals that process large AI responses.
    pub fn new(capacity: usize, raw_capacity: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
            pending: String::new(),
            raw_replay: Vec::new(),
            raw_capacity,
            total_raw_bytes: 0,
        }
    }

    /// Append a complete line to the buffer, dropping the oldest if at capacity.
    pub fn push_line(&mut self, line: String) {
        if self.capacity == 0 {
            return;
        }
        if self.lines.len() >= self.capacity {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Append text that may contain newlines and/or incomplete trailing lines.
    ///
    /// Complete lines (terminated by `\n`) are committed to the ring buffer immediately.
    /// Any trailing text after the last newline is held in `pending` and will be
    /// prepended to the next chunk.
    pub fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let combined = if self.pending.is_empty() {
            text.to_string()
        } else {
            let mut s = std::mem::take(&mut self.pending);
            s.push_str(text);
            s
        };

        if let Some(last_newline) = combined.rfind('\n') {
            let complete = &combined[..last_newline];
            let remainder = &combined[last_newline + 1..];

            for line in complete.split('\n') {
                self.push_line(line.to_string());
            }

            self.pending = remainder.to_string();
        } else {
            self.pending = combined;
        }
    }

    /// Force the pending partial line into the buffer as a complete line.
    pub fn flush(&mut self) {
        if !self.pending.is_empty() {
            let line = std::mem::take(&mut self.pending);
            self.push_line(line);
        }
    }

    /// Get the current pending (incomplete) line, if any.
    pub fn pending(&self) -> &str {
        &self.pending
    }

    /// Return all committed lines as a slice.
    pub fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    /// Drain all committed lines from the buffer, returning them.
    pub fn drain(&mut self) -> Vec<String> {
        self.lines.drain(..).collect()
    }

    /// Search for a pattern in committed lines. Returns matching lines.
    /// Also searches the pending line.
    pub fn search(&self, pattern: &str) -> Vec<&str> {
        let mut results: Vec<&str> = self
            .lines
            .iter()
            .filter(|line| line.contains(pattern))
            .map(String::as_str)
            .collect();
        if self.pending.contains(pattern) {
            results.push(&self.pending);
        }
        results
    }

    /// Return the number of committed lines in the buffer.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Return true if the buffer has no committed lines and no pending text.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.pending.is_empty()
    }

    /// Get the last N committed lines from the buffer.
    pub fn tail(&self, n: usize) -> Vec<&str> {
        self.lines
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(String::as_str)
            .collect()
    }

    /// Append raw bytes to the replay buffer.
    ///
    /// The raw bytes (ANSI sequences intact) are appended verbatim. When the
    /// buffer would exceed `self.raw_capacity`, the oldest bytes are drained
    /// to make room, keeping the most recent terminal state intact.
    ///
    /// `total_raw_bytes` is always incremented regardless of ring-buffer
    /// trimming, so callers can track cumulative throughput for idle detection.
    pub fn push_raw(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        self.total_raw_bytes += data.len();

        // If the incoming chunk alone exceeds (or equals) capacity,
        // discard everything and keep only the tail of the new data.
        if data.len() >= self.raw_capacity {
            self.raw_replay.clear();
            let start = data.len() - self.raw_capacity;
            self.raw_replay.extend_from_slice(&data[start..]);
            return;
        }

        let new_len = self.raw_replay.len() + data.len();
        if new_len > self.raw_capacity {
            let drop_count = new_len - self.raw_capacity;
            self.raw_replay.drain(..drop_count);
        }
        self.raw_replay.extend_from_slice(data);
    }

    /// Return a copy of the raw replay buffer.
    ///
    /// Contains all raw PTY bytes (with ANSI escape sequences) accumulated
    /// since the terminal started, up to `self.raw_capacity`. Feed these
    /// bytes directly to xterm.js `write()` to restore the terminal state.
    pub fn raw_replay(&self) -> &[u8] {
        &self.raw_replay
    }

    /// Return the number of bytes currently in the raw replay ring buffer.
    ///
    /// This value plateaus at `raw_capacity` once the buffer is full.
    /// For idle detection, use `total_raw_bytes()` instead.
    pub fn raw_len(&self) -> usize {
        self.raw_replay.len()
    }

    /// Return the cumulative count of all raw bytes ever pushed.
    ///
    /// Unlike `raw_len()`, this counter never decreases and is not capped by
    /// the ring-buffer capacity. The orchestration idle detector uses this to
    /// distinguish "buffer full but still receiving" from "truly idle".
    pub fn total_raw_bytes(&self) -> usize {
        self.total_raw_bytes
    }

    /// Clear all committed lines, pending text, raw replay bytes, and
    /// cumulative byte counter.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.pending.clear();
        self.raw_replay.clear();
        self.total_raw_bytes = 0;
    }
}
