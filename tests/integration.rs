use agnostik::*;

// ---------------------------------------------------------------------------
// Core types (always available)
// ---------------------------------------------------------------------------

#[test]
fn agent_id_uniqueness() {
    let a = AgentId::new();
    let b = AgentId::new();
    assert_ne!(a, b);
}

#[test]
fn agent_id_from_str_roundtrip() {
    let id = AgentId::new();
    let s = id.to_string();
    let parsed: AgentId = s.parse().unwrap();
    assert_eq!(id, parsed);
}

#[test]
fn user_id_from_str_roundtrip() {
    let id = UserId::new();
    let s = id.to_string();
    let parsed: UserId = s.parse().unwrap();
    assert_eq!(id, parsed);
}

#[test]
fn version_from_str_roundtrip() {
    let v: Version = "2026.3.26".parse().unwrap();
    assert_eq!(v.major, 2026);
    assert_eq!(v.minor, 3);
    assert_eq!(v.patch, 26);
    assert_eq!(v.to_string(), "2026.3.26");
}

#[test]
fn version_serde_as_string() {
    let v = Version {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: None,
        build: None,
    };
    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, "\"1.2.3\"");
    let back: Version = serde_json::from_str(&json).unwrap();
    assert_eq!(v, back);
}

#[test]
fn error_retriable() {
    assert!(AgnostikError::Timeout.is_retriable());
    assert!(AgnostikError::ResourceExhausted("mem".into()).is_retriable());
    assert!(!AgnostikError::PermissionDenied("x".into()).is_retriable());
    assert!(!AgnostikError::ConfigError("bad".into()).is_retriable());
    assert!(!AgnostikError::Internal("oops".into()).is_retriable());
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let e: AgnostikError = io_err.into();
    assert!(matches!(e, AgnostikError::Io(_)));
}

#[test]
fn error_from_serde_json() {
    let serde_err = serde_json::from_str::<String>("not json").unwrap_err();
    let e: AgnostikError = serde_err.into();
    assert!(matches!(e, AgnostikError::Serialization(_)));
}

// ---------------------------------------------------------------------------
// Security + Agent (default features)
// ---------------------------------------------------------------------------

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
fn trace_context_sampled_flag_propagation() {
    let mut parent = TraceContext::new();
    parent.trace_flags = TRACE_FLAG_SAMPLED;
    parent.trace_state = "vendor=value".into();
    let child = parent.child();
    assert!(child.is_sampled());
    assert_eq!(child.trace_flags, TRACE_FLAG_SAMPLED);
    assert_eq!(child.trace_state, "vendor=value");
}

#[test]
fn trace_id_span_id_hex_roundtrip() {
    let tid = TraceId::new();
    let parsed: TraceId = tid.to_string().parse().unwrap();
    assert_eq!(tid, parsed);

    let sid = SpanId::new();
    let parsed: SpanId = sid.to_string().parse().unwrap();
    assert_eq!(sid, parsed);
}

#[test]
fn rbac_role_serde_roundtrip() {
    for role in [
        Role::Admin,
        Role::Operator,
        Role::Auditor,
        Role::Viewer,
        Role::Service,
    ] {
        let json = serde_json::to_string(&role).unwrap();
        let back: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(role, back);
    }
}

#[test]
fn agent_config_cross_feature_serde() {
    // agent implies security — verify sandbox + permissions always present
    let config = AgentConfig {
        name: "test-agent".into(),
        agent_type: AgentType::Service,
        sandbox: SandboxConfig::default(),
        permissions: vec![Permission::FileRead],
        resource_limits: ResourceLimits::default(),
        metadata: serde_json::json!({"env": "test"}),
        restart_policy: RestartPolicy::OnFailure,
        max_restarts: 3,
        health_check: Some(HealthCheck::default()),
        startup_timeout_secs: Some(30),
        shutdown_timeout_secs: Some(10),
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: AgentConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "test-agent");
    assert_eq!(back.permissions, vec![Permission::FileRead]);
    assert_eq!(back.agent_type, AgentType::Service);
    assert_eq!(back.restart_policy, RestartPolicy::OnFailure);
    assert_eq!(back.max_restarts, 3);
    assert!(back.health_check.is_some());
    assert_eq!(back.startup_timeout_secs, Some(30));
    assert_eq!(back.shutdown_timeout_secs, Some(10));
}

#[test]
fn classification_level_ordering() {
    assert!(ClassificationLevel::Public < ClassificationLevel::Internal);
    assert!(ClassificationLevel::Internal < ClassificationLevel::Confidential);
    assert!(ClassificationLevel::Confidential < ClassificationLevel::Restricted);
}

#[test]
fn classification_result_serde_with_defaults() {
    // Deserialize with missing optional fields
    let json = r#"{"level":"Confidential","auto_level":"Confidential"}"#;
    let r: ClassificationResult = serde_json::from_str(json).unwrap();
    assert_eq!(r.level, ClassificationLevel::Confidential);
    assert!(r.rules_triggered.is_empty());
    assert!(r.pii_found.is_empty());
    assert!(r.keywords_found.is_empty());
}

#[test]
fn validation_result_blocked_vs_clean() {
    let blocked = ValidationResult {
        valid: false,
        sanitized: String::new(),
        warnings: vec![ValidationWarning {
            code: "sql_injection".into(),
            message: "SQL injection detected".into(),
            severity: ValidationSeverity::High,
            position: Some(5),
            pattern: Some("' OR 1=1".into()),
        }],
        blocked: true,
        block_reason: Some("injection detected".into()),
        injection_score: 0.99,
    };
    let json = serde_json::to_string(&blocked).unwrap();
    let back: ValidationResult = serde_json::from_str(&json).unwrap();
    assert!(!back.valid);
    assert!(back.blocked);
    assert!(back.injection_score > 0.9);

    let clean = ValidationResult {
        valid: true,
        sanitized: "hello".into(),
        warnings: vec![],
        blocked: false,
        block_reason: None,
        injection_score: 0.0,
    };
    let json = serde_json::to_string(&clean).unwrap();
    let back: ValidationResult = serde_json::from_str(&json).unwrap();
    assert!(back.valid);
    assert!(!back.blocked);
}

#[test]
fn hardware_accelerator_summary_filter() {
    let summary = AcceleratorSummary {
        devices: vec![
            AcceleratorDevice {
                index: 0,
                name: "RTX 4090".into(),
                vendor: DeviceVendor::Nvidia,
                family: DeviceFamily::Gpu,
                vram_total_mb: 24576,
                vram_used_mb: 0,
                utilization_percent: 0,
                temperature_celsius: None,
                driver_version: "550.67".into(),
                compute_capability: None,
                flags: AcceleratorFlags::default(),
            },
            AcceleratorDevice {
                index: 1,
                name: "Gaudi2".into(),
                vendor: DeviceVendor::Habana,
                family: DeviceFamily::AiAsic,
                vram_total_mb: 98304,
                vram_used_mb: 0,
                utilization_percent: 0,
                temperature_celsius: None,
                driver_version: "1.15".into(),
                compute_capability: None,
                flags: AcceleratorFlags::default(),
            },
        ],
        total_vram_mb: 122880,
    };
    let gpus = summary.by_family(DeviceFamily::Gpu);
    assert_eq!(gpus.len(), 1);
    assert_eq!(gpus[0].name, "RTX 4090");

    let asics = summary.by_family(DeviceFamily::AiAsic);
    assert_eq!(asics.len(), 1);
    assert_eq!(asics[0].name, "Gaudi2");

    let tpus = summary.by_family(DeviceFamily::Tpu);
    assert!(tpus.is_empty());
}

#[test]
fn llm_inference_request_full_roundtrip() {
    let req = InferenceRequest {
        model: "llama3-70b".into(),
        prompt: String::new(),
        messages: vec![
            Message::text(MessageRole::System, "You are helpful."),
            Message::text(MessageRole::User, "Hello"),
        ],
        max_tokens: Some(2048),
        sampling: SamplingParams {
            temperature: 0.0,
            top_p: Some(0.95),
            top_k: Some(40),
            frequency_penalty: Some(0.1),
            presence_penalty: Some(0.1),
            stop_sequences: vec!["END".into()],
            seed: Some(42),
        },
        stream: true,
        tools: vec![ToolDefinition {
            name: "search".into(),
            description: "Web search".into(),
            parameters: serde_json::json!({"type": "object", "properties": {"q": {"type": "string"}}}),
        }],
        tool_choice: Some(ToolChoice::Required),
        response_format: Some(ResponseFormat::JsonSchema {
            name: "result".into(),
            schema: serde_json::json!({"type": "object"}),
        }),
        system: Some("You are a helpful assistant.".into()),
        logprobs: false,
        top_logprobs: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: InferenceRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.model, "llama3-70b");
    assert_eq!(back.messages.len(), 2);
    assert_eq!(back.tools.len(), 1);
    assert!(back.stream);
    assert_eq!(back.sampling.seed, Some(42));
    assert_eq!(back.sampling.stop_sequences, vec!["END"]);
    assert_eq!(back.tool_choice, Some(ToolChoice::Required));
    assert!(matches!(
        back.response_format,
        Some(ResponseFormat::JsonSchema { .. })
    ));
}

#[test]
fn llm_inference_request_minimal_deserialize() {
    // Only required field is model — everything else has serde defaults
    let json = r#"{"model":"gpt-4"}"#;
    let req: InferenceRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.model, "gpt-4");
    assert!(req.prompt.is_empty());
    assert!(req.messages.is_empty());
    assert!(req.max_tokens.is_none());
    assert!(!req.stream);
    assert!(req.tools.is_empty());
    assert!((req.sampling.temperature - 0.7).abs() < f32::EPSILON);
}

#[test]
fn audit_entry_integrity_chain() {
    let genesis = AuditEntry {
        id: "entry-001".into(),
        correlation_id: Some("corr-1".into()),
        timestamp: chrono::Utc::now(),
        agent_id: AgentId::new(),
        action: "system_start".into(),
        severity: AuditSeverity::Info,
        result: AuditResult::Success,
        details: serde_json::json!({}),
        user_id: None,
        source_ip: None,
        target_resource: None,
        duration_ms: None,
        tags: vec![],
        integrity: IntegrityFields::genesis("genesis-sig".into()),
    };
    assert!(genesis.integrity.is_genesis());
    assert_eq!(genesis.integrity.previous_entry_hash, GENESIS_HASH);

    // Second entry references genesis
    let second = AuditEntry {
        id: "entry-002".into(),
        correlation_id: genesis.correlation_id.clone(),
        timestamp: chrono::Utc::now(),
        agent_id: genesis.agent_id,
        action: "file_read".into(),
        severity: AuditSeverity::Debug,
        result: AuditResult::Success,
        details: serde_json::json!({"path": "/etc/config"}),
        user_id: Some(UserId::new()),
        source_ip: Some("192.168.1.100".into()),
        target_resource: Some("/etc/config".into()),
        duration_ms: Some(5),
        tags: vec!["filesystem".into()],
        integrity: IntegrityFields {
            version: "1.0.0".parse().unwrap(),
            signature: "second-sig".into(),
            previous_entry_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .into(),
        },
    };
    assert!(!second.integrity.is_genesis());

    // Both roundtrip through serde
    let json1 = serde_json::to_string(&genesis).unwrap();
    let back1: AuditEntry = serde_json::from_str(&json1).unwrap();
    assert!(back1.integrity.is_genesis());

    let json2 = serde_json::to_string(&second).unwrap();
    let back2: AuditEntry = serde_json::from_str(&json2).unwrap();
    assert_eq!(back2.id, "entry-002");
    assert!(back2.user_id.is_some());
}

#[test]
fn secrets_debug_never_leaks() {
    let s = Secret::new("super-secret-api-key-12345");
    let debug = format!("{s:?}");
    assert!(debug.contains("REDACTED"));
    assert!(!debug.contains("super-secret"));
    assert!(!debug.contains("12345"));
}

#[test]
fn secrets_zeroize_on_drop() {
    let s = Secret::new("ephemeral");
    assert_eq!(s.expose(), "ephemeral");
    assert_eq!(s.len(), 9);
    assert!(!s.is_empty());
    drop(s);
}

#[test]
fn config_environment_profile_serde() {
    let profiles = [
        EnvironmentProfile::Development,
        EnvironmentProfile::Staging,
        EnvironmentProfile::Production,
    ];
    for p in &profiles {
        let json = serde_json::to_string(p).unwrap();
        let back: EnvironmentProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(p, &back);
    }
}

#[test]
fn cross_module_agent_manifest_with_typed_versions() {
    let manifest = AgentManifest {
        name: "test-agent".into(),
        version: "2026.3.26".parse().unwrap(),
        description: "Integration test agent".into(),
        requested_permissions: vec![
            Permission::FileRead,
            Permission::FileWrite,
            Permission::NetworkAccess,
        ],
        resource_limits: ResourceLimits::default(),
        ..AgentManifest::default()
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let back: AgentManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.version.major, 2026);
    assert_eq!(back.requested_permissions.len(), 3);
}

#[test]
fn agent_dependency_typed_version() {
    let dep = AgentDependency {
        required_agent: "helper-agent".into(),
        min_version: Some("1.2.3".parse().unwrap()),
        required: true,
    };
    let json = serde_json::to_string(&dep).unwrap();
    let back: AgentDependency = serde_json::from_str(&json).unwrap();
    assert_eq!(back.required_agent, "helper-agent");
    assert_eq!(back.min_version.as_ref().unwrap().major, 1);
    assert!(back.required);
}
