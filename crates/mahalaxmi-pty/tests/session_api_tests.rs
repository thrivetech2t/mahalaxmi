// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Regression tests that lock down the public API surface used by the
//! mahalaxmi-pty example programs (examples/pty/01-spawn-process.rs and
//! examples/pty/02-stream-output.rs).
//!
//! All tests exercise only the crate's public API.  No private modules are
//! imported.

use bytes::Bytes;
use mahalaxmi_core::config::MahalaxmiConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TerminalId, TerminalState};
use mahalaxmi_pty::{
    OutputBuffer, TerminalEvent, TerminalSessionManager, VtCleaner,
    DEFAULT_RAW_REPLAY_CAPACITY_BYTES,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn en_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// OutputBuffer
// ---------------------------------------------------------------------------

#[test]
fn output_buffer_new_is_empty() {
    let buf = OutputBuffer::new(100, 1024);
    assert!(buf.is_empty(), "A freshly constructed OutputBuffer should be empty");
}

#[test]
fn output_buffer_push_line_and_len() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_line("alpha".to_string());
    buf.push_line("beta".to_string());
    buf.push_line("gamma".to_string());
    assert_eq!(buf.len(), 3);
}

#[test]
fn output_buffer_lines_returns_all_pushed() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_line("a".to_string());
    buf.push_line("b".to_string());
    buf.push_line("c".to_string());
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert!(lines.contains(&"a"), "Expected 'a' in lines");
    assert!(lines.contains(&"b"), "Expected 'b' in lines");
    assert!(lines.contains(&"c"), "Expected 'c' in lines");
}

#[test]
fn output_buffer_drain_returns_and_clears() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_line("first".to_string());
    buf.push_line("second".to_string());
    let drained = buf.drain();
    assert_eq!(drained.len(), 2, "drain() should return 2 lines");
    assert!(buf.is_empty(), "Buffer should be empty after drain()");
}

#[test]
fn output_buffer_tail_returns_last_n() {
    let mut buf = OutputBuffer::new(100, 1024);
    for i in 0..5 {
        buf.push_line(format!("line-{}", i));
    }
    let tail = buf.tail(3);
    assert_eq!(tail.len(), 3, "tail(3) should return exactly 3 entries");
}

#[test]
fn output_buffer_search_finds_match() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_line("hello world".to_string());
    let results = buf.search("world");
    assert_eq!(results.len(), 1, "search('world') should find 1 result");
}

#[test]
fn output_buffer_search_no_match_returns_empty() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_line("hello".to_string());
    let results = buf.search("zzz");
    assert!(results.is_empty(), "search('zzz') should return no results");
}

#[test]
fn output_buffer_push_text_assembles_lines_on_newline() {
    let mut buf = OutputBuffer::new(100, 1024);
    buf.push_text("foo\nbar\n");
    // Both complete lines should be committed
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert!(lines.contains(&"foo"), "Expected 'foo' in committed lines");
    assert!(lines.contains(&"bar"), "Expected 'bar' in committed lines");
}

#[test]
fn output_buffer_raw_replay_capacity_constant_is_512kb() {
    assert_eq!(
        DEFAULT_RAW_REPLAY_CAPACITY_BYTES,
        512 * 1024,
        "DEFAULT_RAW_REPLAY_CAPACITY_BYTES should be exactly 512 KB"
    );
}

// ---------------------------------------------------------------------------
// VtCleaner
// ---------------------------------------------------------------------------

#[test]
fn vt_cleaner_new_is_stateful() {
    // Two independent instances should not share state.
    let mut a = VtCleaner::new();
    let mut b = VtCleaner::new();
    let ra = a.clean(b"hello");
    let rb = b.clean(b"world");
    assert_eq!(ra, "hello");
    assert_eq!(rb, "world");
}

#[test]
fn vt_cleaner_clean_plain_text_unchanged() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"hello world");
    assert!(
        result.contains("hello world"),
        "Plain text should pass through unchanged; got: {:?}",
        result
    );
}

#[test]
fn vt_cleaner_strips_ansi_color_codes() {
    let mut cleaner = VtCleaner::new();
    let input = b"\x1b[32mGreen\x1b[0m plain";
    let result = cleaner.clean(input);
    assert!(result.contains("Green"), "Expected 'Green' in cleaned output");
    assert!(result.contains("plain"), "Expected 'plain' in cleaned output");
    assert!(
        !result.contains('\x1b'),
        "Cleaned output should not contain escape characters"
    );
}

#[test]
fn vt_cleaner_strips_bold_sequence() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"\x1b[1mBold\x1b[0m");
    assert_eq!(result.trim(), "Bold", "Expected 'Bold' after stripping bold sequence");
}

#[test]
fn vt_cleaner_preserves_newlines() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"line1\nline2");
    assert!(result.contains("line1"), "Expected 'line1' in output");
    assert!(result.contains("line2"), "Expected 'line2' in output");
}

#[test]
fn vt_cleaner_empty_input_returns_empty() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"");
    assert_eq!(result, "", "Empty input should yield empty output");
}

#[test]
fn vt_cleaner_only_escapes_returns_empty_or_whitespace() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"\x1b[32m\x1b[0m");
    assert!(
        result.trim().is_empty(),
        "Escape-only input should yield empty or whitespace-only output; got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// TerminalEvent construction
// ---------------------------------------------------------------------------

#[test]
fn terminal_event_output_received_is_constructible() {
    let id = TerminalId::new();
    let event = TerminalEvent::OutputReceived {
        terminal_id: id,
        data: Bytes::from("test"),
    };
    match event {
        TerminalEvent::OutputReceived { terminal_id, data } => {
            assert_eq!(terminal_id, id);
            assert_eq!(data.as_ref(), b"test");
        }
        _ => panic!("Expected OutputReceived variant"),
    }
}

#[test]
fn terminal_event_text_output_is_constructible() {
    let id = TerminalId::new();
    let event = TerminalEvent::TextOutput {
        terminal_id: id,
        text: "hello".to_string(),
    };
    match event {
        TerminalEvent::TextOutput { terminal_id, text } => {
            assert_eq!(terminal_id, id);
            assert_eq!(text, "hello");
        }
        _ => panic!("Expected TextOutput variant"),
    }
}

#[test]
fn terminal_event_process_exited_is_constructible() {
    let id = TerminalId::new();
    let event = TerminalEvent::ProcessExited {
        terminal_id: id,
        exit_code: 0,
    };
    match event {
        TerminalEvent::ProcessExited {
            terminal_id,
            exit_code,
        } => {
            assert_eq!(terminal_id, id);
            assert_eq!(exit_code, 0);
        }
        _ => panic!("Expected ProcessExited variant"),
    }
}

#[test]
fn terminal_event_state_changed_is_constructible() {
    let id = TerminalId::new();
    let event = TerminalEvent::StateChanged {
        terminal_id: id,
        old_state: TerminalState::Created,
        new_state: TerminalState::Running,
    };
    match event {
        TerminalEvent::StateChanged {
            terminal_id,
            old_state,
            new_state,
        } => {
            assert_eq!(terminal_id, id);
            assert_eq!(old_state, TerminalState::Created);
            assert_eq!(new_state, TerminalState::Running);
        }
        _ => panic!("Expected StateChanged variant"),
    }
}

// ---------------------------------------------------------------------------
// TerminalSessionManager
// ---------------------------------------------------------------------------

#[test]
fn terminal_session_manager_new_with_config() {
    let i18n = en_i18n();
    let config = MahalaxmiConfig::default();
    let _mgr = TerminalSessionManager::new(&config.orchestration, i18n);
}

#[tokio::test]
async fn terminal_session_manager_subscribe_returns_receiver() {
    let i18n = en_i18n();
    let config = MahalaxmiConfig::default();
    let mgr = TerminalSessionManager::new(&config.orchestration, i18n);
    let mut event_rx = mgr.subscribe();
    // No events have been emitted, so try_recv() should return Empty.
    let result = event_rx.try_recv();
    assert!(
        matches!(
            result,
            Err(tokio::sync::broadcast::error::TryRecvError::Empty)
        ),
        "Expected TryRecvError::Empty on a fresh subscriber; got: {:?}",
        result
    );
}
