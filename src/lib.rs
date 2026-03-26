//! # Agnostik
//!
//! **Agnostik** (agnostic) — shared types, error handling, and domain primitives
//! for the AGNOS ecosystem.
//!
//! Extracted from `agnos-common` as a standalone crate. Provides the core type
//! vocabulary that all AGNOS components share: agent identity, sandbox config,
//! security policies, telemetry, LLM types, and error handling.
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `agent` | yes | Agent types (AgentId, AgentConfig, AgentStatus, AgentManifest) |
//! | `security` | yes | Security types (SandboxConfig, Permission, NetworkAccess) |
//! | `telemetry` | yes | Telemetry types (TraceContext, Span, TelemetryConfig) |
//! | `audit` | no | Audit chain types (AuditEntry, hash chain) |
//! | `llm` | no | LLM types (InferenceRequest, Provider, TokenUsage) |
//! | `secrets` | no | Secret management types (zeroize-backed) |
//! | `config` | no | Component configuration types |

pub mod error;

#[cfg(feature = "agent")]
pub mod agent;

#[cfg(feature = "security")]
pub mod security;

#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "audit")]
pub mod audit;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "secrets")]
pub mod secrets;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "classification")]
pub mod classification;

#[cfg(feature = "validation")]
pub mod validation;

#[cfg(feature = "hardware")]
pub mod hardware;

pub mod types;

#[cfg(feature = "logging")]
pub mod logging;

// Core re-exports (always available)
pub use error::{AgnostikError, Result};
pub use types::*;

// Feature-gated re-exports
#[cfg(feature = "agent")]
pub use agent::{AgentConfig, AgentManifest, AgentStatus, AgentType};

#[cfg(feature = "security")]
pub use security::{FsAccess, NetworkAccess, Permission, SandboxConfig, SeccompAction};

#[cfg(feature = "telemetry")]
pub use telemetry::{Span, SpanId, TelemetryConfig, TraceContext, TraceId};
