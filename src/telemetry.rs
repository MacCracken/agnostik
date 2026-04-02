use crate::types::AgentId;
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

impl std::str::FromStr for TraceId {
    type Err = crate::error::AgnostikError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        u128::from_str_radix(s, 16).map(Self).map_err(|_| {
            crate::error::AgnostikError::InvalidArgument(format!(
                "invalid trace id: {s} (expected 32 hex digits)"
            ))
        })
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

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl std::str::FromStr for SpanId {
    type Err = crate::error::AgnostikError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(Self).map_err(|_| {
            crate::error::AgnostikError::InvalidArgument(format!(
                "invalid span id: {s} (expected 16 hex digits)"
            ))
        })
    }
}

fn rand_u64() -> u64 {
    uuid::Uuid::new_v4().as_u128() as u64
}

// ---------------------------------------------------------------------------
// Resource (OTel-aligned)
// ---------------------------------------------------------------------------

/// Identifies the entity producing telemetry (OTel Resource semantic conventions).
///
/// Every span, metric, and log record should be associated with a `Resource`
/// describing the service, host, and container that produced it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resource {
    /// Logical service name (e.g., "daimon", "hoosh").
    pub service_name: String,
    /// Service version (SemVer).
    #[serde(default)]
    pub service_version: String,
    /// Unique instance ID (hostname, pod name, container ID).
    #[serde(default)]
    pub service_instance_id: String,
    /// Additional resource attributes (OTel semantic conventions).
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
}

/// Distributed trace context (W3C Trace Context compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    #[serde(default)]
    pub parent_span_id: Option<SpanId>,
    /// W3C trace flags (bit 0 = sampled).
    #[serde(default = "default_trace_flags")]
    pub trace_flags: u8,
    /// W3C trace state (vendor-specific key=value pairs).
    #[serde(default)]
    pub trace_state: String,
}

fn default_trace_flags() -> u8 {
    0x01
}

/// W3C trace flag: sampled.
pub const TRACE_FLAG_SAMPLED: u8 = 0x01;

impl TraceContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            trace_flags: TRACE_FLAG_SAMPLED,
            trace_state: String::new(),
        }
    }

    #[must_use]
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
        }
    }

    /// Whether this context is sampled.
    #[must_use]
    pub fn is_sampled(&self) -> bool {
        self.trace_flags & TRACE_FLAG_SAMPLED != 0
    }

    /// Format as a W3C `traceparent` header value.
    ///
    /// Format: `00-{trace_id}-{parent_id}-{flags}`
    #[must_use]
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Parse a W3C `traceparent` header value.
    ///
    /// Expected format: `{version}-{trace_id}-{parent_id}-{flags}`
    pub fn from_traceparent(header: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return Err(crate::error::AgnostikError::InvalidArgument(format!(
                "invalid traceparent: expected 4 dash-separated parts, got {}",
                parts.len()
            )));
        }
        let trace_id: TraceId = parts[1].parse()?;
        let span_id: SpanId = parts[2].parse()?;
        let trace_flags = u8::from_str_radix(parts[3], 16).map_err(|_| {
            crate::error::AgnostikError::InvalidArgument(format!(
                "invalid traceparent flags: {} (expected 2 hex digits)",
                parts[3]
            ))
        })?;
        Ok(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            trace_flags,
            trace_state: String::new(),
        })
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Span status (OTel-aligned: Unset, Ok, Error).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpanStatus {
    /// Status not explicitly set (default).
    Unset,
    /// Operation completed successfully.
    Ok,
    /// Operation failed.
    Error {
        /// Optional error description.
        #[serde(default)]
        message: String,
    },
}

/// Span kind describing the relationship between the span and its parent (OTel-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum SpanKind {
    /// Default. Internal operation within an application.
    #[default]
    Internal,
    /// Handles an inbound synchronous request.
    Server,
    /// Makes an outbound synchronous request.
    Client,
    /// Initiates an asynchronous request (does not wait for response).
    Producer,
    /// Processes an asynchronous request.
    Consumer,
}

/// A timestamped annotation on a span (OTel span event).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name (e.g., "exception", "message").
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
}

/// A link from one span to another (OTel span link).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
}

/// A completed span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub name: String,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub status: SpanStatus,
    /// Span kind (OTel: Internal, Server, Client, Producer, Consumer).
    #[serde(default)]
    pub kind: SpanKind,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub attributes: std::collections::HashMap<String, String>,
    /// Timestamped events recorded during the span (OTel span events).
    #[serde(default)]
    pub events: Vec<SpanEvent>,
    /// Links to related spans in other traces (OTel span links).
    #[serde(default)]
    pub links: Vec<SpanLink>,
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
    pub agent_id: AgentId,
    pub error: String,
    pub backtrace: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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

// ---------------------------------------------------------------------------
// Metrics (OTel-aligned)
// ---------------------------------------------------------------------------

/// Kind of metric instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MetricKind {
    Counter,
    UpDownCounter,
    Gauge,
    Histogram,
}

/// A metric value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MetricValue {
    Int(i64),
    Float(f64),
    Histogram {
        sum: f64,
        count: u64,
        bounds: Vec<f64>,
        bucket_counts: Vec<u64>,
    },
}

/// Describes a metric instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentDescriptor {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub kind: MetricKind,
}

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub instrument: String,
    pub value: MetricValue,
    pub attributes: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Trait for span export backends.
pub trait SpanCollector: Send + Sync {
    /// Export a batch of completed spans.
    fn export(&self, spans: &[Span]) -> crate::Result<()>;

    /// Flush any buffered spans.
    fn flush(&self) -> crate::Result<()> {
        Ok(())
    }

    /// Shutdown the collector, flushing remaining spans.
    fn shutdown(&self) -> crate::Result<()> {
        self.flush()
    }
}

/// Trait for metric export backends.
pub trait MetricSink: Send + Sync {
    /// Export a batch of metric data points.
    fn export(&self, metrics: &[MetricDataPoint]) -> crate::Result<()>;

    /// Flush any buffered metrics.
    fn flush(&self) -> crate::Result<()> {
        Ok(())
    }
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
        assert_ne!(
            SpanStatus::Ok,
            SpanStatus::Error {
                message: String::new()
            }
        );
        assert_ne!(SpanStatus::Unset, SpanStatus::Ok);
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
        let variants = vec![
            SpanStatus::Unset,
            SpanStatus::Ok,
            SpanStatus::Error {
                message: String::new(),
            },
            SpanStatus::Error {
                message: "connection refused".into(),
            },
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).unwrap();
            let back: SpanStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, &back);
        }
    }

    #[test]
    fn span_kind_serde_roundtrip() {
        for variant in [
            SpanKind::Internal,
            SpanKind::Server,
            SpanKind::Client,
            SpanKind::Producer,
            SpanKind::Consumer,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SpanKind = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn span_kind_default() {
        assert_eq!(SpanKind::default(), SpanKind::Internal);
    }

    #[test]
    fn resource_serde_roundtrip() {
        let r = Resource {
            service_name: "daimon".into(),
            service_version: "0.90.0".into(),
            service_instance_id: "daimon-abc123".into(),
            attributes: [("host.name".into(), "node-01".into())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(back.service_name, "daimon");
        assert_eq!(back.attributes.get("host.name").unwrap(), "node-01");
    }

    #[test]
    fn resource_default() {
        let r = Resource::default();
        assert!(r.service_name.is_empty());
        assert!(r.attributes.is_empty());
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
            kind: SpanKind::Server,
            started_at: chrono::Utc::now(),
            duration_ms: 50,
            attributes: [("key".into(), "value".into())].into_iter().collect(),
            events: vec![SpanEvent {
                name: "exception".into(),
                timestamp: chrono::Utc::now(),
                attributes: [("exception.message".into(), "timeout".into())]
                    .into_iter()
                    .collect(),
            }],
            links: vec![SpanLink {
                trace_id: TraceId::new(),
                span_id: SpanId::new(),
                attributes: Default::default(),
            }],
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-span");
        assert_eq!(back.trace_id, s.trace_id);
        assert_eq!(back.status, SpanStatus::Ok);
        assert_eq!(back.kind, SpanKind::Server);
        assert_eq!(back.duration_ms, 50);
        assert_eq!(back.events.len(), 1);
        assert_eq!(back.events[0].name, "exception");
        assert_eq!(back.links.len(), 1);
    }

    #[test]
    fn span_event_serde_roundtrip() {
        let e = SpanEvent {
            name: "log".into(),
            timestamp: chrono::Utc::now(),
            attributes: [("level".into(), "error".into())].into_iter().collect(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: SpanEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "log");
        assert_eq!(back.attributes.get("level").unwrap(), "error");
    }

    #[test]
    fn span_link_serde_roundtrip() {
        let l = SpanLink {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            attributes: [("link.type".into(), "parent".into())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&l).unwrap();
        let back: SpanLink = serde_json::from_str(&json).unwrap();
        assert_eq!(back.trace_id, l.trace_id);
        assert_eq!(back.span_id, l.span_id);
    }

    #[test]
    fn traceparent_roundtrip() {
        let ctx = TraceContext::new();
        let header = ctx.to_traceparent();
        assert!(header.starts_with("00-"));
        let parts: Vec<&str> = header.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "00");
        assert_eq!(parts[1].len(), 32); // trace_id
        assert_eq!(parts[2].len(), 16); // span_id
        assert_eq!(parts[3], "01"); // sampled

        let parsed = TraceContext::from_traceparent(&header).unwrap();
        assert_eq!(parsed.trace_id, ctx.trace_id);
        assert_eq!(parsed.span_id, ctx.span_id);
        assert_eq!(parsed.trace_flags, ctx.trace_flags);
    }

    #[test]
    fn traceparent_invalid() {
        assert!(TraceContext::from_traceparent("bad").is_err());
        assert!(TraceContext::from_traceparent("00-notahex-0000000000000001-01").is_err());
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
        let id = AgentId::new();
        let r = CrashReport {
            agent_id: id,
            error: "segfault".into(),
            backtrace: Some("frame0\nframe1".into()),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: CrashReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
        assert_eq!(back.error, "segfault");
        assert!(back.backtrace.is_some());
    }

    #[test]
    fn trace_context_w3c_flags() {
        let ctx = TraceContext::new();
        assert!(ctx.is_sampled());
        assert_eq!(ctx.trace_flags, TRACE_FLAG_SAMPLED);
        assert!(ctx.trace_state.is_empty());
    }

    #[test]
    fn trace_context_child_propagates_flags() {
        let mut parent = TraceContext::new();
        parent.trace_flags = 0x00;
        parent.trace_state = "vendor=value".into();
        let child = parent.child();
        assert_eq!(child.trace_flags, 0x00);
        assert!(!child.is_sampled());
        assert_eq!(child.trace_state, "vendor=value");
    }

    #[test]
    fn trace_id_from_str_roundtrip() {
        let id = TraceId::new();
        let s = id.to_string();
        let parsed: TraceId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn trace_id_from_str_invalid() {
        assert!("not-hex".parse::<TraceId>().is_err());
    }

    #[test]
    fn span_id_display_from_str_roundtrip() {
        let id = SpanId::new();
        let s = id.to_string();
        assert_eq!(s.len(), 16);
        let parsed: SpanId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn span_id_from_str_invalid() {
        assert!("zzz".parse::<SpanId>().is_err());
    }

    // --- Metric tests ---

    #[test]
    fn metric_kind_serde_roundtrip() {
        for variant in [
            MetricKind::Counter,
            MetricKind::UpDownCounter,
            MetricKind::Gauge,
            MetricKind::Histogram,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: MetricKind = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn metric_value_int_serde_roundtrip() {
        let v = MetricValue::Int(42);
        let json = serde_json::to_string(&v).unwrap();
        let back: MetricValue = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn metric_value_float_serde_roundtrip() {
        let v = MetricValue::Float(42.5);
        let json = serde_json::to_string(&v).unwrap();
        let back: MetricValue = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, MetricValue::Float(f) if (f - 42.5).abs() < f64::EPSILON));
    }

    #[test]
    fn metric_value_histogram_serde_roundtrip() {
        let v = MetricValue::Histogram {
            sum: 150.0,
            count: 10,
            bounds: vec![10.0, 50.0, 100.0],
            bucket_counts: vec![2, 5, 2, 1],
        };
        let json = serde_json::to_string(&v).unwrap();
        let _back: MetricValue = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn instrument_descriptor_serde_roundtrip() {
        let d = InstrumentDescriptor {
            name: "http_request_duration".into(),
            description: "Duration of HTTP requests".into(),
            unit: "ms".into(),
            kind: MetricKind::Histogram,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: InstrumentDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "http_request_duration");
        assert_eq!(back.kind, MetricKind::Histogram);
    }

    #[test]
    fn metric_data_point_serde_roundtrip() {
        let dp = MetricDataPoint {
            instrument: "requests_total".into(),
            value: MetricValue::Int(100),
            attributes: [("method".into(), "GET".into())].into_iter().collect(),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&dp).unwrap();
        let back: MetricDataPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.instrument, "requests_total");
    }

    #[test]
    fn trace_id_default() {
        let a = TraceId::default();
        let b = TraceId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn span_id_default() {
        let a = SpanId::default();
        let b = SpanId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn trace_context_default() {
        let ctx = TraceContext::default();
        assert!(ctx.is_sampled());
    }

    #[test]
    fn trace_context_serde_with_defaults() {
        // Deserialize with only required fields to exercise serde defaults
        let json = r#"{"trace_id":1,"span_id":1}"#;
        let ctx: TraceContext = serde_json::from_str(json).unwrap();
        assert_eq!(ctx.trace_flags, TRACE_FLAG_SAMPLED); // default_trace_flags()
        assert!(ctx.trace_state.is_empty());
    }

    // --- Trait default method coverage ---

    struct NoopCollector;

    impl SpanCollector for NoopCollector {
        fn export(&self, _spans: &[Span]) -> crate::Result<()> {
            Ok(())
        }
    }

    struct NoopSink;

    impl MetricSink for NoopSink {
        fn export(&self, _metrics: &[MetricDataPoint]) -> crate::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn span_collector_default_methods() {
        let c = NoopCollector;
        assert!(c.flush().is_ok());
        assert!(c.shutdown().is_ok());
    }

    #[test]
    fn metric_sink_default_methods() {
        let s = NoopSink;
        assert!(s.flush().is_ok());
    }
}
