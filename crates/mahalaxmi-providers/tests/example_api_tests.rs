// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Integration tests that lock down the public API surface used by the
// mahalaxmi-providers example programs:
//   examples/providers/01-implement-provider.rs
//   examples/providers/02-claude-code-stub.rs
//   examples/providers/03-custom-cli-provider.rs

use mahalaxmi_core::config::MahalaxmiConfig;
use mahalaxmi_providers::{
    credentials::{AuthMode, CredentialSpec},
    metadata::ProviderMetadata,
    traits::AiProvider,
    types::{CostTier, ProviderCapabilities, TaskType},
    ClaudeCodeProvider, CustomCliProvider,
};
use std::path::Path;

// ---------------------------------------------------------------------------
// ClaudeCodeProvider::new()
// ---------------------------------------------------------------------------

#[test]
fn claude_new_creates_provider() {
    let _provider = ClaudeCodeProvider::new();
}

#[test]
fn claude_from_mahalaxmi_config_creates_provider() {
    let config = MahalaxmiConfig::default();
    let _provider = ClaudeCodeProvider::from_mahalaxmi_config(&config);
}

#[test]
fn claude_name_is_non_empty() {
    let provider = ClaudeCodeProvider::new();
    assert!(!provider.name().is_empty());
}

#[test]
fn claude_id_is_claude_code() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.id().as_str(), "claude-code");
}

#[test]
fn claude_cli_binary_is_claude() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.cli_binary(), "claude");
}

// ---------------------------------------------------------------------------
// ProviderCapabilities for ClaudeCodeProvider
// ---------------------------------------------------------------------------

#[test]
fn claude_caps_supports_streaming_true() {
    let provider = ClaudeCodeProvider::new();
    assert!(provider.capabilities().supports_streaming);
}

#[test]
fn claude_caps_supports_agent_teams_true() {
    let provider = ClaudeCodeProvider::new();
    assert!(provider.capabilities().supports_agent_teams);
}

#[test]
fn claude_caps_max_context_tokens_200k() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.capabilities().max_context_tokens, 200_000);
}

#[test]
fn claude_caps_cost_tier_is_high() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.capabilities().cost_tier, CostTier::High);
}

#[test]
fn claude_caps_supports_local_only_false() {
    let provider = ClaudeCodeProvider::new();
    assert!(!provider.capabilities().supports_local_only);
}

#[test]
fn claude_caps_supports_tool_use_true() {
    let provider = ClaudeCodeProvider::new();
    assert!(provider.capabilities().supports_tool_use);
}

#[test]
fn claude_caps_routing_score_code_generation_gt_zero() {
    let provider = ClaudeCodeProvider::new();
    assert!(provider.capabilities().routing_score(TaskType::CodeGeneration) > 0);
}

// ---------------------------------------------------------------------------
// build_command
// ---------------------------------------------------------------------------

#[test]
fn claude_build_command_returns_ok() {
    let provider = ClaudeCodeProvider::new();
    let result = provider.build_command(Path::new("/tmp"), "test prompt");
    assert!(result.is_ok(), "build_command returned Err: {:?}", result.err());
}

#[test]
fn claude_build_command_program_is_claude() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/tmp"), "test prompt")
        .expect("build_command should succeed");
    assert_eq!(cmd.program, "claude");
}

#[test]
fn claude_build_command_args_contain_prompt() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/tmp"), "my unique prompt string")
        .expect("build_command should succeed");
    let args_str = cmd.args.join(" ");
    assert!(
        args_str.contains("my unique prompt string"),
        "args did not contain prompt; got: {args_str:?}"
    );
}

#[test]
fn claude_build_command_stdin_data_is_none() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/tmp"), "test")
        .expect("build_command should succeed");
    assert!(cmd.stdin_data.is_none());
}

// ---------------------------------------------------------------------------
// credential_requirements
// ---------------------------------------------------------------------------

#[test]
fn claude_credential_requirements_non_empty() {
    let provider = ClaudeCodeProvider::new();
    let reqs = provider.credential_requirements();
    assert!(!reqs.is_empty(), "credential_requirements should be non-empty");
}

// ---------------------------------------------------------------------------
// output_markers
// ---------------------------------------------------------------------------

#[test]
fn claude_output_markers_completion_marker_non_empty() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(!markers.completion_marker.as_str().is_empty());
}

#[test]
fn claude_output_markers_error_marker_non_empty() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(!markers.error_marker.as_str().is_empty());
}

#[test]
fn claude_output_markers_prompt_marker_non_empty() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(!markers.prompt_marker.as_str().is_empty());
}

// ---------------------------------------------------------------------------
// stream_complete_marker — must not panic
// ---------------------------------------------------------------------------

#[test]
fn claude_stream_complete_marker_does_not_panic() {
    let provider = ClaudeCodeProvider::new();
    let _ = provider.stream_complete_marker();
}

// ---------------------------------------------------------------------------
// CustomCliProvider
// ---------------------------------------------------------------------------

#[test]
fn custom_cli_new_creates_provider() {
    let _provider = CustomCliProvider::new();
}

#[test]
fn custom_cli_from_mahalaxmi_config_creates_provider() {
    let config = MahalaxmiConfig::default();
    let _provider = CustomCliProvider::from_mahalaxmi_config(&config);
}

// ---------------------------------------------------------------------------
// ProviderMetadata builder API
// ---------------------------------------------------------------------------

#[test]
fn provider_metadata_new_sets_install_hint() {
    let meta = ProviderMetadata::new("install me");
    assert_eq!(meta.install_hint, "install me");
}

#[test]
fn provider_metadata_with_auth_mode_none_adds_one_mode() {
    let meta = ProviderMetadata::new("hint").with_auth_mode(AuthMode::None);
    assert_eq!(meta.auth_modes.len(), 1);
}

#[test]
fn provider_metadata_with_test_args_works() {
    let meta =
        ProviderMetadata::new("hint").with_test_args(vec!["--version".to_string()]);
    assert_eq!(meta.test_args, vec!["--version"]);
}

// ---------------------------------------------------------------------------
// AuthMode variants are constructible
// ---------------------------------------------------------------------------

#[test]
fn auth_mode_none_is_constructible() {
    let _mode = AuthMode::None;
}

#[test]
fn auth_mode_api_key_is_constructible() {
    let _mode = AuthMode::ApiKey {
        env_var: "MY_KEY".to_string(),
    };
}

#[test]
fn auth_mode_cli_login_is_constructible() {
    let _mode = AuthMode::CliLogin {
        login_command: "x".to_string(),
        check_command: "y".to_string(),
    };
}

// ---------------------------------------------------------------------------
// ProviderCapabilities::default
// ---------------------------------------------------------------------------

#[test]
fn provider_capabilities_default_supports_streaming_true() {
    let caps = ProviderCapabilities::default();
    assert!(caps.supports_streaming);
}

#[test]
fn provider_capabilities_routing_score_general_gt_zero() {
    let caps = ProviderCapabilities::default();
    // Default has no task_proficiency entries → falls back to Proficiency::Good (score=2)
    // cost bonus = 4 - 2 (Medium) = 2; context bonus = 0 (max_context_tokens == 0)
    // score = 2*10 + 2 + 0 = 22
    assert!(caps.routing_score(TaskType::General) > 0);
}

// ---------------------------------------------------------------------------
// configure / clone_box via ClaudeCodeProvider (dyn AiProvider methods)
// ---------------------------------------------------------------------------

#[test]
fn claude_configure_returns_box_with_same_name() {
    let provider = ClaudeCodeProvider::new();
    let config = MahalaxmiConfig::default();
    let configured: Box<dyn AiProvider> = provider.configure(&config);
    assert_eq!(configured.name(), provider.name());
}

#[test]
fn claude_clone_box_returns_box_with_same_name() {
    let provider = ClaudeCodeProvider::new();
    let cloned: Box<dyn AiProvider> = provider.clone_box();
    assert_eq!(cloned.name(), provider.name());
}

// ---------------------------------------------------------------------------
// CostTier variants all exist and have distinct weight() values
// ---------------------------------------------------------------------------

#[test]
fn cost_tier_variants_exist_and_weights_are_ordered() {
    assert_eq!(CostTier::Free.weight(), 0);
    assert_eq!(CostTier::Low.weight(), 1);
    assert_eq!(CostTier::Medium.weight(), 2);
    assert_eq!(CostTier::High.weight(), 3);
    assert_eq!(CostTier::Premium.weight(), 4);

    // Each tier's weight is strictly less than the next.
    assert!(CostTier::Free.weight() < CostTier::Low.weight());
    assert!(CostTier::Low.weight() < CostTier::Medium.weight());
    assert!(CostTier::Medium.weight() < CostTier::High.weight());
    assert!(CostTier::High.weight() < CostTier::Premium.weight());
}
