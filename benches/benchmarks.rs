use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_agent_id_new(c: &mut Criterion) {
    c.bench_function("agent/agent_id_new", |b| {
        b.iter(agnostik::AgentId::new);
    });
}

fn bench_trace_context_new(c: &mut Criterion) {
    c.bench_function("telemetry/trace_context_new", |b| {
        b.iter(agnostik::TraceContext::new);
    });
}

fn bench_trace_context_child(c: &mut Criterion) {
    let parent = agnostik::TraceContext::new();
    c.bench_function("telemetry/trace_context_child", |b| {
        b.iter(|| black_box(&parent).child());
    });
}

fn bench_sandbox_config_default(c: &mut Criterion) {
    c.bench_function("security/sandbox_config_default", |b| {
        b.iter(agnostik::SandboxConfig::default);
    });
}

// ---------------------------------------------------------------------------
// Serde benchmarks
// ---------------------------------------------------------------------------

fn bench_trace_context_serde(c: &mut Criterion) {
    let ctx = agnostik::TraceContext::new();
    let json = serde_json::to_string(&ctx).unwrap();

    c.bench_function("serde/trace_context_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&ctx)).unwrap());
    });
    c.bench_function("serde/trace_context_deserialize", |b| {
        b.iter(|| serde_json::from_str::<agnostik::TraceContext>(black_box(&json)).unwrap());
    });
}

fn bench_sandbox_config_serde(c: &mut Criterion) {
    let cfg = agnostik::SandboxConfig::default();
    let json = serde_json::to_string(&cfg).unwrap();

    c.bench_function("serde/sandbox_config_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&cfg)).unwrap());
    });
    c.bench_function("serde/sandbox_config_deserialize", |b| {
        b.iter(|| serde_json::from_str::<agnostik::SandboxConfig>(black_box(&json)).unwrap());
    });
}

fn bench_agent_id_serde(c: &mut Criterion) {
    let id = agnostik::AgentId::new();
    let json = serde_json::to_string(&id).unwrap();

    c.bench_function("serde/agent_id_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&id)).unwrap());
    });
    c.bench_function("serde/agent_id_deserialize", |b| {
        b.iter(|| serde_json::from_str::<agnostik::AgentId>(black_box(&json)).unwrap());
    });
}

// ---------------------------------------------------------------------------
// LLM serde benchmarks
// ---------------------------------------------------------------------------

fn bench_inference_request_serde(c: &mut Criterion) {
    use agnostik::llm::*;

    let req = InferenceRequest {
        model: "llama3-70b".into(),
        prompt: String::new(),
        messages: vec![
            Message::text(MessageRole::System, "You are a helpful assistant."),
            Message::text(
                MessageRole::User,
                "Explain quantum computing in simple terms.",
            ),
        ],
        max_tokens: Some(1024),
        sampling: SamplingParams::default(),
        stream: false,
        tools: vec![ToolDefinition {
            name: "search".into(),
            description: "Search the web".into(),
            parameters: serde_json::json!({"type": "object", "properties": {"query": {"type": "string"}}}),
        }],
        tool_choice: None,
        response_format: None,
        system: None,
        logprobs: false,
        top_logprobs: None,
    };
    let json = serde_json::to_string(&req).unwrap();

    c.bench_function("serde/inference_request_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&req)).unwrap());
    });
    c.bench_function("serde/inference_request_deserialize", |b| {
        b.iter(|| serde_json::from_str::<InferenceRequest>(black_box(&json)).unwrap());
    });
}

// ---------------------------------------------------------------------------
// Audit serde benchmarks
// ---------------------------------------------------------------------------

fn bench_audit_entry_serde(c: &mut Criterion) {
    use agnostik::audit::*;

    let entry = AuditEntry {
        id: "entry-001".into(),
        correlation_id: Some("corr-abc".into()),
        timestamp: chrono::Utc::now(),
        agent_id: agnostik::AgentId::new(),
        action: "file_read".into(),
        severity: AuditSeverity::Info,
        result: AuditResult::Success,
        details: serde_json::json!({"path": "/tmp/test", "bytes": 4096}),
        user_id: Some(agnostik::UserId::new()),
        source_ip: Some("10.0.0.1".into()),
        target_resource: Some("/tmp/test".into()),
        duration_ms: Some(12),
        tags: vec!["filesystem".into()],
        integrity: IntegrityFields::genesis("sig-abc123".into()),
    };
    let json = serde_json::to_string(&entry).unwrap();

    c.bench_function("serde/audit_entry_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&entry)).unwrap());
    });
    c.bench_function("serde/audit_entry_deserialize", |b| {
        b.iter(|| serde_json::from_str::<AuditEntry>(black_box(&json)).unwrap());
    });
}

// ---------------------------------------------------------------------------
// Hardware serde benchmarks
// ---------------------------------------------------------------------------

fn bench_accelerator_device_serde(c: &mut Criterion) {
    use agnostik::hardware::*;

    let device = AcceleratorDevice {
        index: 0,
        name: "NVIDIA RTX 4090".into(),
        vendor: DeviceVendor::Nvidia,
        family: DeviceFamily::Gpu,
        vram_total_mb: 24576,
        vram_used_mb: 4096,
        utilization_percent: 45,
        temperature_celsius: Some(72.0),
        driver_version: "550.67".into(),
        compute_capability: Some("8.9".into()),
        flags: AcceleratorFlags {
            cuda_available: true,
            ..AcceleratorFlags::default()
        },
    };
    let json = serde_json::to_string(&device).unwrap();

    c.bench_function("serde/accelerator_device_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&device)).unwrap());
    });
    c.bench_function("serde/accelerator_device_deserialize", |b| {
        b.iter(|| serde_json::from_str::<AcceleratorDevice>(black_box(&json)).unwrap());
    });
}

criterion_group!(
    benches,
    bench_agent_id_new,
    bench_trace_context_new,
    bench_trace_context_child,
    bench_sandbox_config_default,
    bench_trace_context_serde,
    bench_sandbox_config_serde,
    bench_agent_id_serde,
    bench_inference_request_serde,
    bench_audit_entry_serde,
    bench_accelerator_device_serde,
);
criterion_main!(benches);
