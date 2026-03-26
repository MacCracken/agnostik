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

criterion_group!(
    benches,
    bench_agent_id_new,
    bench_trace_context_new,
    bench_trace_context_child,
    bench_sandbox_config_default,
    bench_trace_context_serde,
    bench_sandbox_config_serde,
    bench_agent_id_serde,
);
criterion_main!(benches);
