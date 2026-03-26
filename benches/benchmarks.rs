use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_id_new(c: &mut Criterion) {
    c.bench_function("agent/agent_id_new", |b| {
        b.iter(|| agnostik::AgentId::new());
    });
}

fn bench_trace_context_new(c: &mut Criterion) {
    c.bench_function("telemetry/trace_context_new", |b| {
        b.iter(|| agnostik::TraceContext::new());
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
        b.iter(|| agnostik::SandboxConfig::default());
    });
}

criterion_group!(benches, bench_agent_id_new, bench_trace_context_new, bench_trace_context_child, bench_sandbox_config_default);
criterion_main!(benches);
