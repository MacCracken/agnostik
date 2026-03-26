use serde::{Deserialize, Serialize};

/// Filesystem access level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FsAccess {
    NoAccess,
    ReadOnly,
    ReadWrite,
}

/// Network access level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NetworkAccess {
    None,
    LocalhostOnly,
    Restricted,
    Full,
}

/// Seccomp filter action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeccompAction {
    Allow,
    Deny,
    Trap,
}

/// Agent permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Permission {
    FileRead,
    FileWrite,
    NetworkAccess,
    ProcessSpawn,
    LlmInference,
    AuditRead,
}

/// Filesystem access rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemRule {
    pub path: std::path::PathBuf,
    pub access: FsAccess,
}

/// Seccomp rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompRule {
    pub syscall: String,
    pub action: SeccompAction,
}

/// Per-agent network firewall policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub allowed_outbound_ports: Vec<u16>,
    pub allowed_outbound_hosts: Vec<String>,
    pub allowed_inbound_ports: Vec<u16>,
    pub enable_nat: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            allowed_outbound_ports: vec![80, 443],
            allowed_outbound_hosts: Vec::new(),
            allowed_inbound_ports: Vec::new(),
            enable_nat: true,
        }
    }
}

/// Encrypted storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedStorageConfig {
    pub enabled: bool,
    pub size_mb: u64,
    pub filesystem: String,
}

impl Default for EncryptedStorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size_mb: 256,
            filesystem: "ext4".into(),
        }
    }
}

/// Sandbox configuration for agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub filesystem_rules: Vec<FilesystemRule>,
    pub network_access: NetworkAccess,
    pub seccomp_rules: Vec<SeccompRule>,
    pub isolate_network: bool,
    #[serde(default)]
    pub network_policy: Option<NetworkPolicy>,
    #[serde(default)]
    pub mac_profile: Option<String>,
    #[serde(default)]
    pub encrypted_storage: Option<EncryptedStorageConfig>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            filesystem_rules: vec![FilesystemRule {
                path: "/tmp".into(),
                access: FsAccess::ReadWrite,
            }],
            network_access: NetworkAccess::LocalhostOnly,
            seccomp_rules: vec![],
            isolate_network: true,
            network_policy: None,
            mac_profile: None,
            encrypted_storage: None,
        }
    }
}

/// Security context for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub agent_id: String,
    pub permissions: Vec<Permission>,
    pub sandbox: SandboxConfig,
}

/// Security policy effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PolicyEffect {
    Allow,
    Deny,
    Audit,
}

/// Security policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub effect: PolicyEffect,
    pub permissions: Vec<Permission>,
}

/// Security capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Capability {
    Landlock,
    Seccomp,
    Namespaces,
    Cgroups,
    Tpm,
    SecureBoot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_config_default() {
        let c = SandboxConfig::default();
        assert_eq!(c.network_access, NetworkAccess::LocalhostOnly);
        assert!(c.isolate_network);
        assert_eq!(c.filesystem_rules.len(), 1);
    }

    #[test]
    fn network_policy_default() {
        let p = NetworkPolicy::default();
        assert_eq!(p.allowed_outbound_ports, vec![80, 443]);
        assert!(p.enable_nat);
    }

    #[test]
    fn encrypted_storage_default() {
        let e = EncryptedStorageConfig::default();
        assert!(!e.enabled);
        assert_eq!(e.size_mb, 256);
    }

    #[test]
    fn sandbox_config_serde() {
        let c = SandboxConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: SandboxConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.network_access, NetworkAccess::LocalhostOnly);
    }

    #[test]
    fn permission_variants() {
        assert_ne!(Permission::FileRead, Permission::FileWrite);
        assert_ne!(Permission::NetworkAccess, Permission::LlmInference);
    }

    #[test]
    fn fs_access_variants() {
        assert_ne!(FsAccess::NoAccess, FsAccess::ReadOnly);
    }

    #[test]
    fn policy_effect_variants() {
        assert_ne!(PolicyEffect::Allow, PolicyEffect::Deny);
    }

    #[test]
    fn fs_access_serde_roundtrip() {
        for variant in [FsAccess::NoAccess, FsAccess::ReadOnly, FsAccess::ReadWrite] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FsAccess = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn network_access_serde_roundtrip() {
        for variant in [
            NetworkAccess::None,
            NetworkAccess::LocalhostOnly,
            NetworkAccess::Restricted,
            NetworkAccess::Full,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: NetworkAccess = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn seccomp_action_serde_roundtrip() {
        for variant in [
            SeccompAction::Allow,
            SeccompAction::Deny,
            SeccompAction::Trap,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SeccompAction = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn permission_serde_roundtrip() {
        for variant in [
            Permission::FileRead,
            Permission::FileWrite,
            Permission::NetworkAccess,
            Permission::ProcessSpawn,
            Permission::LlmInference,
            Permission::AuditRead,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: Permission = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn filesystem_rule_serde_roundtrip() {
        let r = FilesystemRule {
            path: "/tmp".into(),
            access: FsAccess::ReadWrite,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: FilesystemRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, std::path::PathBuf::from("/tmp"));
        assert_eq!(back.access, FsAccess::ReadWrite);
    }

    #[test]
    fn seccomp_rule_serde_roundtrip() {
        let r = SeccompRule {
            syscall: "write".into(),
            action: SeccompAction::Allow,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: SeccompRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.syscall, "write");
        assert_eq!(back.action, SeccompAction::Allow);
    }

    #[test]
    fn network_policy_serde_roundtrip() {
        let p = NetworkPolicy::default();
        let json = serde_json::to_string(&p).unwrap();
        let back: NetworkPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back.allowed_outbound_ports, vec![80, 443]);
        assert_eq!(back.enable_nat, p.enable_nat);
    }

    #[test]
    fn encrypted_storage_config_serde_roundtrip() {
        let e = EncryptedStorageConfig::default();
        let json = serde_json::to_string(&e).unwrap();
        let back: EncryptedStorageConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.enabled, e.enabled);
        assert_eq!(back.size_mb, e.size_mb);
        assert_eq!(back.filesystem, e.filesystem);
    }

    #[test]
    fn security_context_serde_roundtrip() {
        let ctx = SecurityContext {
            agent_id: "agent-001".into(),
            permissions: vec![Permission::FileRead, Permission::NetworkAccess],
            sandbox: SandboxConfig::default(),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let back: SecurityContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "agent-001");
        assert_eq!(back.permissions.len(), 2);
    }

    #[test]
    fn policy_effect_serde_roundtrip() {
        for variant in [PolicyEffect::Allow, PolicyEffect::Deny, PolicyEffect::Audit] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: PolicyEffect = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn security_policy_serde_roundtrip() {
        let p = SecurityPolicy {
            name: "default".into(),
            effect: PolicyEffect::Allow,
            permissions: vec![Permission::FileRead],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: SecurityPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "default");
        assert_eq!(back.effect, PolicyEffect::Allow);
    }

    #[test]
    fn capability_serde_roundtrip() {
        for variant in [
            Capability::Landlock,
            Capability::Seccomp,
            Capability::Namespaces,
            Capability::Cgroups,
            Capability::Tpm,
            Capability::SecureBoot,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: Capability = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }
}
