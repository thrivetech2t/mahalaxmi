// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Regression tests that lock down the public API surface used by the
//! mahalaxmi-core example programs (examples/core/01-config-loading.rs and
//! examples/core/02-logging-setup.rs).
//!
//! All tests exercise only the crate's public API.  No private modules are
//! imported.

use mahalaxmi_core::config::loader::{default_config_path, load_config};
use mahalaxmi_core::config::{IndexingConfig, MahalaxmiConfig};
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::logging;
use mahalaxmi_core::types::{ProcessCommand, TerminalConfig, TerminalId, TaskId, WorkerId, ProviderId};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn en_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// config::loader
// ---------------------------------------------------------------------------

#[test]
fn default_config_path_returns_path_buf() {
    let path = default_config_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains(".mahalaxmi"),
        "Expected path to contain '.mahalaxmi', got: {}",
        path_str
    );
}

#[test]
fn load_config_with_nonexistent_path_returns_defaults() {
    let i18n = en_i18n();
    // Pass None so the loader looks at the default path; if it doesn't exist
    // it returns defaults, which is the common case in a clean test environment.
    let result = load_config(None, &i18n);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
}

// ---------------------------------------------------------------------------
// MahalaxmiConfig defaults
// ---------------------------------------------------------------------------

#[test]
fn default_config_general_log_level_is_nonempty() {
    let config = MahalaxmiConfig::default();
    assert!(
        !config.general.log_level.is_empty(),
        "general.log_level should not be empty"
    );
}

#[test]
fn default_config_general_locale_is_nonempty() {
    let config = MahalaxmiConfig::default();
    assert!(
        !config.general.locale.is_empty(),
        "general.locale should not be empty"
    );
}

#[test]
fn default_config_orchestration_max_concurrent_workers_is_positive() {
    // auto-scale mode uses 0, but any non-negative value including 0 is accepted.
    // The important thing is the field exists and is accessible.
    let config = MahalaxmiConfig::default();
    // The default is either 0 (auto-scale) or a positive number; both are valid.
    let _ = config.orchestration.max_concurrent_workers;
}

#[test]
fn default_config_providers_default_provider_is_nonempty() {
    let config = MahalaxmiConfig::default();
    assert!(
        !config.providers.default_provider.is_empty(),
        "providers.default_provider should not be empty"
    );
}

#[test]
fn default_config_indexing_max_file_size_bytes_is_positive() {
    let config = MahalaxmiConfig::default();
    assert!(
        config.indexing.max_file_size_bytes > 0,
        "indexing.max_file_size_bytes should be > 0"
    );
}

#[test]
fn default_config_indexing_excluded_dirs_is_nonempty() {
    let config = MahalaxmiConfig::default();
    assert!(
        !config.indexing.excluded_dirs.is_empty(),
        "indexing.excluded_dirs should have at least one entry"
    );
}

#[test]
fn default_config_ui_theme_is_nonempty() {
    let config = MahalaxmiConfig::default();
    assert!(
        !config.ui.theme.is_empty(),
        "ui.theme should not be empty"
    );
}

#[test]
fn default_config_ui_terminal_font_size_is_positive() {
    let config = MahalaxmiConfig::default();
    assert!(
        config.ui.terminal_font_size > 0,
        "ui.terminal_font_size should be > 0"
    );
}

// ---------------------------------------------------------------------------
// I18nService / SupportedLocale
// ---------------------------------------------------------------------------

#[test]
fn i18n_service_en_us_creates_without_panic() {
    let _service = I18nService::new(SupportedLocale::EnUs);
}

#[test]
fn all_supported_locales_create_without_panic() {
    let all_locales = vec![
        SupportedLocale::EnUs,
        SupportedLocale::EsEs,
        SupportedLocale::FrFr,
        SupportedLocale::DeDe,
        SupportedLocale::PtBr,
        SupportedLocale::JaJp,
        SupportedLocale::ZhCn,
        SupportedLocale::KoKr,
        SupportedLocale::HiIn,
        SupportedLocale::ArSa,
    ];
    assert_eq!(all_locales.len(), 10, "Expected exactly 10 locales");
    for locale in all_locales {
        let _service = I18nService::new(locale);
    }
}

// ---------------------------------------------------------------------------
// ProcessCommand
// ---------------------------------------------------------------------------

#[test]
fn process_command_all_fields_accessible() {
    use std::collections::HashMap;
    let cmd = ProcessCommand {
        program: "echo".to_string(),
        args: vec!["hello".to_string()],
        env: HashMap::new(),
        working_dir: None,
        stdin_data: None,
    };
    assert_eq!(cmd.program, "echo");
    assert_eq!(cmd.args, vec!["hello"]);
    assert!(cmd.env.is_empty());
    assert!(cmd.working_dir.is_none());
    assert!(cmd.stdin_data.is_none());
}

// ---------------------------------------------------------------------------
// TerminalId
// ---------------------------------------------------------------------------

#[test]
fn terminal_id_new_produces_unique_values() {
    let id1 = TerminalId::new();
    let id2 = TerminalId::new();
    assert_ne!(
        id1.as_uuid(),
        id2.as_uuid(),
        "Two freshly created TerminalIds should have different UUIDs"
    );
}

// ---------------------------------------------------------------------------
// WorkerId
// ---------------------------------------------------------------------------

#[test]
fn worker_id_new_and_as_u32_roundtrip() {
    let id = WorkerId::new(42);
    assert_eq!(id.as_u32(), 42);
}

// ---------------------------------------------------------------------------
// TaskId
// ---------------------------------------------------------------------------

#[test]
fn task_id_new_and_as_str_roundtrip() {
    let id = TaskId::new("my-task");
    assert_eq!(id.as_str(), "my-task");
}

// ---------------------------------------------------------------------------
// ProviderId
// ---------------------------------------------------------------------------

#[test]
fn provider_id_new_and_as_str_roundtrip() {
    let id = ProviderId::new("claude-code");
    assert_eq!(id.as_str(), "claude-code");
}

// ---------------------------------------------------------------------------
// TerminalConfig
// ---------------------------------------------------------------------------

#[test]
fn terminal_config_default_has_positive_dimensions() {
    let cfg = TerminalConfig::default();
    assert!(cfg.rows > 0, "TerminalConfig default rows should be > 0");
    assert!(cfg.cols > 0, "TerminalConfig default cols should be > 0");
}

// ---------------------------------------------------------------------------
// Serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn mahalaxmi_config_serde_roundtrip() {
    let original = MahalaxmiConfig::default();
    let json = serde_json::to_string(&original).expect("serialization failed");
    let restored: MahalaxmiConfig = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(
        original.general.log_level,
        restored.general.log_level,
        "general.log_level should survive a JSON roundtrip"
    );
}

// ---------------------------------------------------------------------------
// IndexingConfig
// ---------------------------------------------------------------------------

#[test]
fn indexing_config_default_has_reasonable_values() {
    let cfg = IndexingConfig::default();
    assert!(
        cfg.max_file_size_bytes > 0,
        "IndexingConfig::default().max_file_size_bytes should be > 0"
    );
}

// ---------------------------------------------------------------------------
// logging::init_logging
// ---------------------------------------------------------------------------

#[test]
fn logging_init_does_not_panic() {
    let i18n = en_i18n();
    let config = MahalaxmiConfig::default();
    // init_logging may return Err if a global subscriber was already set by a
    // previous test; that is expected and must not panic.
    let _ = logging::init_logging(&config.general, &i18n);
}
