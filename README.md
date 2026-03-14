Mahalaxmi runs a coordinated team of AI coding agents in parallel,
with a consensus engine that reviews and reconciles their work —
so you get the speed of parallelism without the chaos.

[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)
[![Build](https://github.com/thrivetech2t/mahalaxmi/actions/workflows/ci.yml/badge.svg)](https://github.com/thrivetech2t/mahalaxmi/actions)
[![Discord](https://img.shields.io/discord/bSkzhTPK?label=Discord&color=5865F2)](https://discord.gg/bSkzhTPK)

![Mahalaxmi Dashboard](docs/assets/demo.gif)
<!-- TODO: replace with actual demo GIF before launch -->

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

Mahalaxmi uses a Manager-Worker architecture:

- The **Manager** analyzes your codebase, breaks work into tasks,
  and produces an execution plan with dependency ordering
- **Workers** execute tasks in parallel inside isolated PTY sessions,
  each running a real AI CLI tool (Claude Code, OpenAI, Copilot, etc.)
- The **Consensus Engine** reviews all worker outputs, detects
  conflicts, and produces a unified diff for your approval

You review and approve. Mahalaxmi handles the rest.

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

## Community

- 💬 [Discord](https://discord.gg/bSkzhTPK) — questions, showcase, provider plugin help
- 🐦 [Twitter](https://x.com/MahalaxmiDev) — releases and updates
- 🐛 [Issues](https://github.com/thrivetech2t/mahalaxmi/issues) — bug reports
- 💡 [Discussions](https://github.com/thrivetech2t/mahalaxmi/discussions) — feature ideas

## Contributing

Community contributions are welcome in these areas:
- Provider plugins — implement the `AiProvider` trait for a new tool
- Detection rules — add state detection patterns
- Language parsers — add Tree-sitter grammar support
- Bug fixes and documentation

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
A [CLA](CLA.md) is required before your first PR is merged.

## License

The foundation crates in this repository are MIT licensed.
See [LICENSE](LICENSE) for details.

The full Mahalaxmi product — including the orchestration driver,
GraphRAG engine, cloud service, and desktop app — is proprietary
software available at [mahalaxmi.ai](https://mahalaxmi.ai).

Mahalaxmi™ is a trademark of ThriveTech Services LLC.
