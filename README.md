# Agnostik

**Agnostik** (agnostic) — shared types, error handling, and domain primitives for the [AGNOS](https://github.com/MacCracken/agnosticos) ecosystem.

Extracted from `agnos-common` as a standalone crate. Provides the core type vocabulary that all AGNOS components share.

## Features

- **Agent** — AgentId, AgentConfig, AgentStatus, AgentManifest, ResourceLimits, AgentEvent
- **Security** — SandboxConfig, Permission, NetworkPolicy, CgroupLimits, NamespaceConfig, LandlockRuleset, LinuxCapability, CapabilitySet
- **Telemetry** — TraceContext (W3C-compatible), Span, MetricKind, MetricDataPoint, SpanCollector/MetricSink traits
- **Audit** — AuditEntry, AuditSeverity, hash chain types
- **LLM** — Message, ContentBlock, ToolDefinition, ToolCall, SamplingParams, InferenceRequest/Response, StreamEvent
- **Secrets** — Zeroize-backed Secret (redacted Debug), SecretMetadata
- **Config** — EnvironmentProfile, AgnosConfig, ComponentConfig
- **Error** — AgnostikError with 11 variants, retriable classification, From<io::Error>/From<serde_json::Error>

## Quick Start

```rust
use agnostik::{AgentId, SandboxConfig, TraceContext, AgnostikError};

let id = AgentId::new();
let sandbox = SandboxConfig::default();
let trace = TraceContext::new();
let child = trace.child(); // inherits trace_id, flags, state
assert!(trace.is_sampled());

// Parse from strings
let parsed: AgentId = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `agent` | yes | Agent identity, configuration, and lifecycle types (implies `security`) |
| `security` | yes | Sandbox, permission, cgroup, namespace, landlock, capability types |
| `telemetry` | yes | Distributed tracing, metrics, SpanCollector/MetricSink traits |
| `audit` | no | Audit chain entry types |
| `llm` | no | LLM conversation, tool calling, streaming, and inference types |
| `secrets` | no | Zeroize-backed secret storage |
| `config` | no | Environment profile and component config |
| `logging` | no | Tracing subscriber initialization |
| `full` | no | All features enabled |

## Consumers

Every AGNOS component: daimon, hoosh, agnoshi, aegis, argonaut, sigil, ark, kavach, stiva, nein, and all consumer apps.

## License

GPL-3.0-only
