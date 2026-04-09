# Agnostik

**Agnostik** (agnostic) — shared types, error handling, and domain primitives for the [AGNOS](https://github.com/MacCracken/agnosticos) ecosystem.

Written in [Cyrius](https://github.com/MacCracken/cyrius). Zero external dependencies. Ported from Rust (7,121 lines) to Cyrius (~3,200 lines) — conversion complete, Rust source removed.

## Modules

- **Error** — AgnostikError with 11 variants, retriable classification, numeric error codes
- **Types** — AgentId, UserId, Version (SemVer), Capabilities, MessageType, SystemStatus
- **Agent** — AgentConfig, AgentStatus, AgentManifest, ResourceLimits, HealthCheck, LifecycleHooks, AgentPool
- **Security** — SandboxConfig, RBAC (Role, TokenPayload, AuthContext), CgroupLimits, NamespaceConfig, LandlockRuleset, LinuxCapability, CapabilitySet, SeccompProfile
- **Telemetry** — TraceContext (W3C), Span, MetricDataPoint, LogRecord, SpanCollector/MetricSink traits
- **Audit** — AuditEntry with HMAC-SHA256 integrity chain, AuditSeverity, AuditSink trait
- **LLM** — Message, ContentBlock, ToolDefinition, ToolCall, SamplingParams, InferenceRequest/Response, StreamEvent, ModelCapabilities
- **Secrets** — Zeroize-backed Secret, SecretMetadata, SecretStore trait
- **Config** — EnvironmentProfile, AgnosConfig, EdgeResourceOverrides, FleetConfig
- **Classification** — ClassificationLevel, PiiKind (16 variants), ClassificationResult
- **Validation** — ValidationResult, ValidationWarning, InjectionScores (SQL/XSS/command/path/prompt)
- **Hardware** — AcceleratorDevice, DeviceFamily, DeviceVendor, AcceleratorFlags, AcceleratorSummary

## Quick Start

```cyrius
include "src/lib.cyr"

var id = agent_id_new();
var sb = sandbox_config_new();
var ctx = trace_context_new();
var child = tctx_child(ctx);    # inherits trace_id, flags, state
assert_eq(tctx_is_sampled(ctx), 1, "sampled");

# Parse from strings
var parsed = agent_id_from_str(str_from("550e8400-e29b-41d4-a716-446655440000"));
assert_eq(is_ok(parsed), 1, "valid UUID");
```

## Consumers

Every AGNOS component: daimon, hoosh, agnoshi, aegis, argonaut, sigil, ark, kavach, stiva, nein, and all consumer apps.

## License

GPL-3.0-only
