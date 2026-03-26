use agnostik::*;

#[test]
fn agent_id_uniqueness() {
    let a = AgentId::new();
    let b = AgentId::new();
    assert_ne!(a, b);
}

#[test]
fn sandbox_config_defaults() {
    let c = SandboxConfig::default();
    assert_eq!(c.network_access, NetworkAccess::LocalhostOnly);
    assert!(c.isolate_network);
}

#[test]
fn trace_context_parent_child() {
    let parent = TraceContext::new();
    let child = parent.child();
    assert_eq!(parent.trace_id, child.trace_id);
    assert_eq!(child.parent_span_id, Some(parent.span_id));
}

#[test]
fn error_retriable() {
    assert!(AgnostikError::Timeout.is_retriable());
    assert!(!AgnostikError::PermissionDenied("x".into()).is_retriable());
}
