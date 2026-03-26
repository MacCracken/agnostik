# Agnostik

**Agnostik** (agnostic) — shared types, error handling, and domain primitives for the [AGNOS](https://github.com/MacCracken/agnosticos) ecosystem.

Extracted from `agnos-common` as a standalone crate. Provides the core type vocabulary that all AGNOS components share.

## Features

- **Agent** — AgentId, AgentConfig, AgentStatus, AgentManifest, ResourceLimits
- **Security** — SandboxConfig, Permission, NetworkAccess, NetworkPolicy, SeccompAction
- **Telemetry** — TraceContext (W3C-compatible), TraceId, SpanId, Span, EventType
- **Audit** — AuditEntry, AuditSeverity, hash chain types
- **LLM** — InferenceRequest/Response, TokenUsage, LlmProvider, FinishReason
- **Secrets** — Zeroize-backed Secret, SecretMetadata
- **Config** — EnvironmentProfile, AgnosConfig, ComponentConfig
- **Error** — AgnostikError with retriable classification

## Quick Start

```rust
use agnostik::{AgentId, SandboxConfig, TraceContext, AgnostikError};

let id = AgentId::new();
let sandbox = SandboxConfig::default();
let trace = TraceContext::new();
let child = trace.child(); // inherits trace_id
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `agent` | yes | Agent identity and configuration types |
| `security` | yes | Sandbox, permission, network policy types |
| `telemetry` | yes | Distributed tracing types |
| `audit` | no | Audit chain entry types |
| `llm` | no | LLM inference request/response types |
| `secrets` | no | Zeroize-backed secret storage |
| `config` | no | Environment profile and component config |

## Consumers

Every AGNOS component: daimon, hoosh, agnoshi, aegis, argonaut, sigil, ark, kavach, stiva, nein, and all consumer apps.

## License

GPL-3.0
