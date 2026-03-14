// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! Domain area discovery for adversarial manager deliberation.

/// A discrete domain area identified for adversarial deliberation.
///
/// Each `DomainArea` represents a coherent slice of the codebase or product
/// (e.g. "Authentication", "Database Layer", "Frontend UI") that should be
/// deliberated over independently by the adversarial manager team.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainArea {
    /// Short human-readable name (e.g. `"Authentication"`).
    pub name: String,
    /// Brief description of the domain's scope and boundaries.
    pub description: String,
}
