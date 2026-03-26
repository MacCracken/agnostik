use serde::{Deserialize, Serialize};

/// Trace identifier (128-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub u128);

impl TraceId {
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().as_u128())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// Span identifier (64-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(pub u64);

impl SpanId {
    #[must_use]
    pub fn new() -> Self {
        Self(rand_u64())
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_u64() -> u64 {
    uuid::Uuid::new_v4().as_u128() as u64
}

/// Distributed trace context (W3C Trace Context compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    #[serde(default)]
    pub parent_span_id: Option<SpanId>,
}

impl TraceContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
        }
    }

    #[must_use]
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Span status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpanStatus {
    Ok,
    Error,
    Cancelled,
}

/// A completed span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub name: String,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub status: SpanStatus,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub attributes: std::collections::HashMap<String, String>,
}

/// Telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub export_endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 1.0,
            export_endpoint: None,
        }
    }
}

/// Crash report for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub agent_id: String,
    pub error: String,
    pub backtrace: Option<String>,
    pub timestamp: String,
}

/// Event types for pub/sub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventType {
    AgentStarted,
    AgentStopped,
    AgentFailed,
    InferenceComplete,
    AuditEvent,
    SecurityAlert,
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_id_unique() {
        let a = TraceId::new();
        let b = TraceId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn trace_id_display_hex() {
        let t = TraceId::new();
        let s = format!("{t}");
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn trace_context_child_shares_trace() {
        let parent = TraceContext::new();
        let child = parent.child();
        assert_eq!(parent.trace_id, child.trace_id);
        assert_ne!(parent.span_id, child.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn telemetry_config_default() {
        let c = TelemetryConfig::default();
        assert!(c.enabled);
        assert!((c.sample_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn span_status_variants() {
        assert_ne!(SpanStatus::Ok, SpanStatus::Error);
    }

    #[test]
    fn event_type_variants() {
        assert_ne!(EventType::AgentStarted, EventType::AgentStopped);
    }

    #[test]
    fn trace_context_serde() {
        let ctx = TraceContext::new();
        let json = serde_json::to_string(&ctx).unwrap();
        let back: TraceContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx.trace_id, back.trace_id);
    }

    #[test]
    fn span_status_serde_roundtrip() {
        for variant in [SpanStatus::Ok, SpanStatus::Error, SpanStatus::Cancelled] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SpanStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn event_type_serde_roundtrip() {
        for variant in [
            EventType::AgentStarted,
            EventType::AgentStopped,
            EventType::AgentFailed,
            EventType::InferenceComplete,
            EventType::AuditEvent,
            EventType::SecurityAlert,
            EventType::Custom,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EventType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn span_serde_roundtrip() {
        let s = Span {
            name: "test-span".into(),
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            status: SpanStatus::Ok,
            start_ms: 1000,
            duration_ms: 50,
            attributes: [("key".into(), "value".into())].into_iter().collect(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-span");
        assert_eq!(back.trace_id, s.trace_id);
        assert_eq!(back.status, SpanStatus::Ok);
        assert_eq!(back.duration_ms, 50);
    }

    #[test]
    fn telemetry_config_serde_roundtrip() {
        let c = TelemetryConfig {
            enabled: true,
            sample_rate: 0.5,
            export_endpoint: Some("http://localhost:4317".into()),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert!(back.enabled);
        assert!((back.sample_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            back.export_endpoint.as_deref(),
            Some("http://localhost:4317")
        );
    }

    #[test]
    fn crash_report_serde_roundtrip() {
        let r = CrashReport {
            agent_id: "agent-001".into(),
            error: "segfault".into(),
            backtrace: Some("frame0\nframe1".into()),
            timestamp: "2026-03-25T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: CrashReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "agent-001");
        assert_eq!(back.error, "segfault");
        assert!(back.backtrace.is_some());
    }
}
