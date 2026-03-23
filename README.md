# Mahalaxmi

**Parallel AI coding agents with consensus — run Claude Code, Copilot, and Ollama side-by-side,
then let the consensus engine reconcile their work into a single unified diff.**

Mahalaxmi is an open-source Rust orchestration engine that spawns multiple AI coding agents
in isolated PTY sessions, coordinates them via a Manager-Worker architecture,
and uses a consensus protocol to detect conflicts and produce a clean, reviewable result.
No chaos. No merge hell. Just parallel speed with a human-in-the-loop safety net.

[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)
[![Build](https://github.com/mahalaxmi-ai/mahalaxmi/actions/workflows/ci.yml/badge.svg)](https://github.com/mahalaxmi-ai/mahalaxmi/actions)
[![Discord](https://img.shields.io/discord/bSkzhTPK?label=Discord&color=5865F2)](https://discord.gg/bSkzhTPK)
[![Spec: Peer Review](https://img.shields.io/badge/MFOP%20Spec-Peer%20Review%20Open-yellow)](specs/README.md)
[![BioMetrics: Patent Pending](https://img.shields.io/badge/BioMetrics-Patent%20Pending%20%E2%80%94%20Open%20for%20Comment-green)](specs/en-US/biometrics.md)

> [!IMPORTANT]
> **The MFOP v1.0 Protocol Specification is open for public peer review.**
> Read it in your language and leave feedback via a GitHub issue — your input directly shapes the protocol.
>
> | 🇺🇸 [English](specs/en-US/mfop-protocol.md) | 🇪🇸 [Español](specs/es-ES/mfop-protocol.md) | 🇫🇷 [Français](specs/fr-FR/mfop-protocol.md) | 🇩🇪 [Deutsch](specs/de-DE/mfop-protocol.md) | 🇮🇳 [हिन्दी](specs/hi-IN/mfop-protocol.md) |
> |---|---|---|---|---|
> | 🇯🇵 [日本語](specs/ja-JP/mfop-protocol.md) | 🇰🇷 [한국어](specs/ko-KR/mfop-protocol.md) | 🇧🇷 [Português](specs/pt-BR/mfop-protocol.md) | 🇨🇳 [中文](specs/zh-CN/mfop-protocol.md) | 🇸🇦 [العربية](specs/ar-SA/mfop-protocol.md) |
>
> → [**Submit feedback via GitHub Issues**](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) · [Full spec index with status](specs/README.md)

> [!NOTE]
> **BioMetrics Patent Pending Disclosure — open for technical comment.**
> A U.S. Provisional Patent Application was filed March 22, 2026. The full technical disclosure is published here for public record and peer review. Comments on the architecture, prior art, or technical claims are welcome via GitHub Issues.
>
> | 🇺🇸 [English](specs/en-US/biometrics.md) | 🇪🇸 [Español](specs/es-ES/biometrics.md) | 🇫🇷 [Français](specs/fr-FR/biometrics.md) | 🇩🇪 [Deutsch](specs/de-DE/biometrics.md) | 🇮🇳 [हिन्दी](specs/hi-IN/biometrics.md) |
> |---|---|---|---|---|
> | 🇯🇵 [日本語](specs/ja-JP/biometrics.md) | 🇰🇷 [한국어](specs/ko-KR/biometrics.md) | 🇧🇷 [Português](specs/pt-BR/biometrics.md) | 🇨🇳 [中文](specs/zh-CN/biometrics.md) | 🇸🇦 [العربية](specs/ar-SA/biometrics.md) |
>
> → [**Submit feedback via GitHub Issues**](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics) · Patent Pending © 2026 ThriveTech Services LLC

![Mahalaxmi Dashboard](docs/assets/demo.gif)
<!-- TODO: replace with actual demo GIF before launch -->

> **Keywords:** AI coding agents · multi-agent orchestration · LLM parallelism · PTY terminal automation ·
> consensus engine · Claude Code · GitHub Copilot · Ollama · multi-provider routing · Rust · agentic coding

## Quick Start

```bash
# 1. Install
cargo install mahalaxmi-cli

# 2. Configure your first provider
mahalaxmi provider add claude-code

# 3. Run your first orchestration cycle
mahalaxmi run
```

## How It Works

Mahalaxmi uses a **Manager-Worker consensus architecture**:

1. The **Manager** analyzes your codebase via AST-based repo maps,
   decomposes the task into a dependency-ordered DAG, and assigns subtasks to workers
2. **Workers** execute in parallel inside isolated PTY sessions —
   each worker drives a real AI CLI tool (Claude Code, Copilot, Codex, Ollama, or any custom CLI)
   over its actual terminal interface, with no SDK wrapping
3. The **Consensus Engine** collects all worker outputs, runs conflict detection
   across file-level and semantic boundaries, and produces a single unified diff
4. **You review and approve** — one clean patch, not five conflicting ones

This means you get genuine multi-provider parallelism (not just one model with retries),
real terminal I/O (not API calls that bypass your auth or tool config),
and a deterministic reconciliation step before anything touches your repo.

## Why PTY-Based Routing

Most multi-agent systems call LLM APIs directly. Mahalaxmi does not.

Instead, each worker spawns your actual AI CLI tool in a pseudo-terminal — the same way
you'd use it interactively. This has meaningful consequences:

- **No API key juggling** — workers authenticate exactly as you do (OAuth, keychain, enterprise SSO)
- **Full tool fidelity** — workers use each tool's native file editing, diff, and search capabilities
- **Provider isolation** — a bug in one provider's output can't corrupt another worker's context
- **Bring any CLI** — if it runs in a terminal and produces diffs, you can plug it in

## Supported Providers

| Provider | Status |
|----------|--------|
| Claude Code (Anthropic) | ✅ Built-in |
| GitHub Copilot | ✅ Built-in |
| OpenAI Codex | ✅ Built-in |
| Ollama (local models) | ✅ Built-in |
| Custom CLI | ✅ Built-in — bring any AI CLI tool |
| Community plugins | [Contribute one →](CONTRIBUTING.md) |

## Repository Structure

| Crate | Description |
|-------|-------------|
| [`mahalaxmi-core`](crates/mahalaxmi-core) | Domain types, config, logging |
| [`mahalaxmi-pty`](crates/mahalaxmi-pty) | PTY spawning and terminal I/O |
| [`mahalaxmi-orchestration`](crates/mahalaxmi-orchestration) | Consensus engine and DAG types |
| [`mahalaxmi-detection`](crates/mahalaxmi-detection) | State detection rule engine |
| [`mahalaxmi-providers`](crates/mahalaxmi-providers) | Provider trait and reference implementations |
| [`mahalaxmi-indexing`](crates/mahalaxmi-indexing) | AST parsing and repo maps |
| [`mahalaxmi-cli`](crates/mahalaxmi-cli) | Command-line interface |

## Build a Provider Plugin

The highest-value contribution you can make is adding a new AI provider.
If your favorite AI CLI tool isn't in the table above, you can add it in one Rust file.

Implement the `AiProvider` trait from `mahalaxmi-providers`.
This skeleton covers every required method — paste it, fill in `build_command`
and `validate_credentials`, and it will compile:

```rust
// In Cargo.toml:
// mahalaxmi-providers = { git = "https://github.com/mahalaxmi-ai/mahalaxmi" }

use async_trait::async_trait;
use mahalaxmi_providers::{
    AiProvider, CredentialSpec, MahalaxmiConfig, MahalaxmiResult,
    OutputMarkers, ProviderCapabilities, ProviderMetadata, ProviderId,
    I18nService, ProcessCommand,
};
use std::path::Path;

#[derive(Clone)]
pub struct MyToolProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    metadata: ProviderMetadata,
}

impl MyToolProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("mytool"),
            capabilities: ProviderCapabilities::default(),
            markers: OutputMarkers::new(
                r"DONE",                      // pattern that signals completion
                r"(?i)(error|fatal|failed)",  // pattern that signals an error
                r">\s*$",                     // pattern that signals waiting for input
            ).expect("markers are valid regex"),
            metadata: ProviderMetadata::new("pip install mytool"), // install hint
        }
    }
}

#[async_trait]
impl AiProvider for MyToolProvider {
    fn name(&self) -> &str { "My Tool" }
    fn id(&self) -> &ProviderId { &self.id }
    fn cli_binary(&self) -> &str { "mytool" }
    fn metadata(&self) -> &ProviderMetadata { &self.metadata }
    fn capabilities(&self) -> &ProviderCapabilities { &self.capabilities }
    fn output_markers(&self) -> &OutputMarkers { &self.markers }
    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![] // return CredentialSpec entries for any API keys / env vars needed
    }

    fn build_command(&self, dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        // Build the shell command that launches your CLI with the prompt.
        // The PTY engine will spawn this command and manage its terminal I/O.
        Ok(ProcessCommand::new(self.cli_binary())
            .arg("--some-flag")
            .arg(prompt)
            .working_dir(dir))
    }

    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        // Check env vars, files, or keyrings. No network calls.
        // Return Err(MahalaxmiError::ProviderNotConfigured { .. }) if credentials are missing.
        Ok(())
    }

    fn configure(&self, _config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}
```

The trait has optional overrides for streaming markers, token extraction, and model
selection — leave them at their defaults until you need them.
See [`crates/mahalaxmi-providers/src/ollama.rs`](crates/mahalaxmi-providers/src/ollama.rs)
for a complete real-world implementation.

**To submit a provider:**
1. Open a [Provider Plugin issue](.github/ISSUE_TEMPLATE/provider_plugin.md) first to claim the slot
2. Fork, implement, add tests, open a PR against `main`
3. Accept the [CLA](CLA.md) via PR comment (`I have read and agree to the CLA.`)

Other contributions (detection rules, language parsers, bug fixes, docs) are also welcome.
See [CONTRIBUTING.md](CONTRIBUTING.md) for the full scope.

## Protocol Specification — Open for Peer Review

The **Mahalaxmi Federation and Orchestration Protocol (MFOP) v1.0** specification is published here for public peer review.

MFOP defines how Mahalaxmi nodes federate, route jobs, sign billing receipts, and settle economically across heterogeneous compute infrastructure. The spec is in **draft status** — all feedback is welcome: technical corrections, translation accuracy, clarity improvements, or protocol design questions.

**How to comment:** Open a [Specification Feedback issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback). The issue form guides you to reference the section number and language. You can also comment directly on the spec `.md` files via a pull request, or start a thread in [GitHub Discussions](https://github.com/mahalaxmi-ai/mahalaxmi/discussions).

| Language | Read the Spec | Leave Feedback |
|---|---|---|
| 🇺🇸 English (en-US) | [MFOP Protocol v1.0](specs/en-US/mfop-protocol.md) | [Open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-en-US) |
| 🇪🇸 Español (es-ES) | [Protocolo MFOP v1.0](specs/es-ES/mfop-protocol.md) | [Abrir un problema](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-es-ES) |
| 🇫🇷 Français (fr-FR) | [Protocole MFOP v1.0](specs/fr-FR/mfop-protocol.md) | [Ouvrir un ticket](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-fr-FR) |
| 🇩🇪 Deutsch (de-DE) | [MFOP-Protokoll v1.0](specs/de-DE/mfop-protocol.md) | [Issue erstellen](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-de-DE) |
| 🇮🇳 हिन्दी (hi-IN) | [MFOP प्रोटोकॉल v1.0](specs/hi-IN/mfop-protocol.md) | [समस्या खोलें](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-hi-IN) |
| 🇯🇵 日本語 (ja-JP) | [MFOP プロトコル v1.0](specs/ja-JP/mfop-protocol.md) | [Issue を開く](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-ja-JP) |
| 🇰🇷 한국어 (ko-KR) | [MFOP 프로토콜 v1.0](specs/ko-KR/mfop-protocol.md) | [이슈 열기](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-ko-KR) |
| 🇧🇷 Português (pt-BR) | [Protocolo MFOP v1.0](specs/pt-BR/mfop-protocol.md) | [Abrir uma issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-pt-BR) |
| 🇨🇳 中文 (zh-CN) | [MFOP 协议 v1.0](specs/zh-CN/mfop-protocol.md) | [提交 Issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-zh-CN) |
| 🇸🇦 العربية (ar-SA) | [بروتوكول MFOP v1.0](specs/ar-SA/mfop-protocol.md) | [فتح مشكلة](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,lang-ar-SA) |

→ [Full spec index with status and review notes](specs/README.md)

## BioMetrics — Patent Pending Disclosure

The **BioMetrics** patent disclosure describes a federated AI orchestration system with cryptographic chain-of-custody for multi-modal biometric identification workflows. A U.S. Provisional Patent Application was filed **March 22, 2026** with the USPTO. This document is published for public record and peer review.

The invention introduces: a Root Federation Manager with bounded delegation depth; Manager-Worker quorum consensus producing structured identity assertions; cryptographically signed WorkUnitReceipt chain-of-custody; jurisdiction-scoped policy enforcement at the AI inference layer; and federated deduplication without cross-jurisdiction raw biometric transmission.

**How to comment:** Open a [BioMetrics Feedback issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics). Reference the section number and language in your issue. Comments on the architecture, prior art distinctions, or technical claims are welcome.

| Language | Read the Disclosure | Leave Feedback |
|---|---|---|
| 🇺🇸 English (en-US) | [BioMetrics Disclosure](specs/en-US/biometrics.md) | [Open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-en-US) |
| 🇪🇸 Español (es-ES) | [Divulgación BioMetrics](specs/es-ES/biometrics.md) | [Abrir un problema](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-es-ES) |
| 🇫🇷 Français (fr-FR) | [Divulgation BioMetrics](specs/fr-FR/biometrics.md) | [Ouvrir un ticket](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-fr-FR) |
| 🇩🇪 Deutsch (de-DE) | [BioMetrics-Offenbarung](specs/de-DE/biometrics.md) | [Issue erstellen](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-de-DE) |
| 🇮🇳 हिन्दी (hi-IN) | [BioMetrics प्रकटीकरण](specs/hi-IN/biometrics.md) | [समस्या खोलें](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-hi-IN) |
| 🇯🇵 日本語 (ja-JP) | [BioMetrics 開示](specs/ja-JP/biometrics.md) | [Issue を開く](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-ja-JP) |
| 🇰🇷 한국어 (ko-KR) | [BioMetrics 공시](specs/ko-KR/biometrics.md) | [이슈 열기](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-ko-KR) |
| 🇧🇷 Português (pt-BR) | [Divulgação BioMetrics](specs/pt-BR/biometrics.md) | [Abrir uma issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-pt-BR) |
| 🇨🇳 中文 (zh-CN) | [BioMetrics 披露](specs/zh-CN/biometrics.md) | [提交 Issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-zh-CN) |
| 🇸🇦 العربية (ar-SA) | [إفصاح BioMetrics](specs/ar-SA/biometrics.md) | [فتح مشكلة](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics,lang-ar-SA) |

Patent Pending. © 2026 ThriveTech Services LLC. All rights reserved.

## Community

- 💬 [Discord](https://discord.gg/bSkzhTPK) — questions, showcase, provider plugin help
- 🐦 [Twitter](https://x.com/MahalaxmiDev) — releases and updates
- 🐛 [Issues](https://github.com/mahalaxmi-ai/mahalaxmi/issues) — bug reports
- 💡 [Discussions](https://github.com/mahalaxmi-ai/mahalaxmi/discussions) — feature ideas

## License

The foundation crates in this repository are MIT licensed.
See [LICENSE](LICENSE) for details.

The full Mahalaxmi product — including the orchestration driver,
GraphRAG engine, cloud service, and desktop app — is proprietary
software available at [mahalaxmi.ai](https://mahalaxmi.ai).

Mahalaxmi™ is a trademark of ThriveTech Services LLC.
