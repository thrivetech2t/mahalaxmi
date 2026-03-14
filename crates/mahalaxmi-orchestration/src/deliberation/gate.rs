// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Configuration gate for the adversarial deliberation protocol.

/// Configuration governing the adversarial multi-agent manager deliberation.
///
/// Controls which models play each deliberation role and whether the
/// adversarial protocol is enabled for a given orchestration cycle.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdversarialDeliberationConfig {
    /// Model identifier for the Proposer and Challenger roles.
    ///
    /// Defaults to `"claude-haiku-4-5-20251001"` — a fast, cost-effective
    /// model suited to generating and critiquing task lists.
    #[serde(default = "default_proposer_model")]
    pub proposer_model: String,

    /// Model identifier for the Synthesizer role.
    ///
    /// Defaults to `"claude-sonnet-4-6"` — a higher-capability model used
    /// to produce the authoritative final task list.
    #[serde(default = "default_synthesizer_model")]
    pub synthesizer_model: String,

    /// Whether the adversarial deliberation protocol is active.
    ///
    /// When `false`, the orchestration engine falls back to standard
    /// single-manager proposal generation.
    #[serde(default)]
    pub enabled: bool,
}

fn default_proposer_model() -> String {
    "claude-haiku-4-5-20251001".to_owned()
}

fn default_synthesizer_model() -> String {
    "claude-sonnet-4-6".to_owned()
}
