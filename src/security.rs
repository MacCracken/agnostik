use crate::types::AgentId;
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

/// Seccomp filter action (OCI runtime spec aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeccompAction {
    Allow,
    /// Kill the thread (SECCOMP_RET_KILL_THREAD).
    Kill,
    /// Kill the process (SECCOMP_RET_KILL_PROCESS).
    KillProcess,
    /// Send SIGSYS (SECCOMP_RET_TRAP).
    Trap,
    /// Return an errno value (SECCOMP_RET_ERRNO).
    Errno(u32),
    /// Notify a tracing process (SECCOMP_RET_TRACE).
    Trace(u32),
    /// Log the syscall (SECCOMP_RET_LOG).
    Log,
}

/// Seccomp argument comparison operator (OCI runtime spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeccompArgOp {
    NotEqual,
    LessThan,
    LessEqual,
    Equal,
    GreaterEqual,
    GreaterThan,
    MaskedEqual,
}

/// A condition on a syscall argument for seccomp filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeccompArg {
    /// Argument index (0-5).
    pub index: u32,
    /// Value to compare against.
    pub value: u64,
    /// Second value (for MaskedEqual: mask).
    #[serde(default)]
    pub value_two: u64,
    /// Comparison operator.
    pub op: SeccompArgOp,
}

/// Target architecture for seccomp filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeccompArch {
    X86,
    X86_64,
    X32,
    Arm,
    Aarch64,
    Mips,
    Mips64,
    Mips64n32,
    Mipsel,
    Mipsel64,
    Mipsel64n32,
    Ppc,
    Ppc64,
    Ppc64le,
    S390,
    S390x,
    Riscv64,
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

/// A seccomp syscall rule with optional argument conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompRule {
    /// Syscall names this rule applies to.
    pub names: Vec<String>,
    pub action: SeccompAction,
    /// Argument conditions (all must match for the action to apply).
    #[serde(default)]
    pub args: Vec<SeccompArg>,
}

/// Complete seccomp filter profile (OCI runtime spec aligned).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompProfile {
    /// Action to take when no rule matches.
    pub default_action: SeccompAction,
    /// Architectures this profile applies to.
    #[serde(default)]
    pub architectures: Vec<SeccompArch>,
    /// Seccomp filter flags.
    #[serde(default)]
    pub flags: Vec<String>,
    /// Syscall rules (evaluated in order).
    #[serde(default)]
    pub syscalls: Vec<SeccompRule>,
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
    /// Full seccomp filter profile (preferred over legacy `seccomp_rules`).
    #[serde(default)]
    pub seccomp: Option<SeccompProfile>,
    pub isolate_network: bool,
    #[serde(default)]
    pub network_policy: Option<NetworkPolicy>,
    /// AppArmor profile name (e.g., "runtime/default").
    #[serde(default)]
    pub apparmor_profile: Option<String>,
    /// SELinux process label (e.g., "system_u:system_r:container_t:s0").
    #[serde(default)]
    pub selinux_label: Option<String>,
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
            seccomp: None,
            isolate_network: true,
            network_policy: None,
            apparmor_profile: None,
            selinux_label: None,
            encrypted_storage: None,
        }
    }
}

/// Security context for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub agent_id: AgentId,
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

/// Platform security features available on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SystemFeature {
    Landlock,
    Seccomp,
    Namespaces,
    Cgroups,
    Tpm,
    SecureBoot,
}

/// Backward compatibility alias.
pub type Capability = SystemFeature;

// ---------------------------------------------------------------------------
// Cgroups v2
// ---------------------------------------------------------------------------

/// Cgroup v2 resource limits for agent sandboxing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CgroupLimits {
    /// Hard memory limit in bytes (`memory.max`).
    #[serde(default)]
    pub memory_max: Option<u64>,
    /// Soft memory limit in bytes (`memory.high`).
    #[serde(default)]
    pub memory_high: Option<u64>,
    /// CPU bandwidth: max microseconds per period (`cpu.max`).
    #[serde(default)]
    pub cpu_max_usec: Option<u64>,
    /// CPU bandwidth period in microseconds (default 100000).
    #[serde(default)]
    pub cpu_period_usec: Option<u64>,
    /// CPU weight 1-10000 (`cpu.weight`, default 100).
    #[serde(default)]
    pub cpu_weight: Option<u16>,
    /// Max number of PIDs (`pids.max`).
    #[serde(default)]
    pub pids_max: Option<u32>,
}

// ---------------------------------------------------------------------------
// Namespaces
// ---------------------------------------------------------------------------

/// Which Linux namespaces to unshare for an agent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceConfig {
    #[serde(default)]
    pub pid: bool,
    #[serde(default)]
    pub net: bool,
    #[serde(default)]
    pub mount: bool,
    #[serde(default)]
    pub user: bool,
    #[serde(default)]
    pub uts: bool,
    #[serde(default)]
    pub ipc: bool,
    #[serde(default)]
    pub cgroup: bool,
}

/// UID/GID mapping for user namespaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdMapping {
    pub inside_id: u32,
    pub outside_id: u32,
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Landlock
// ---------------------------------------------------------------------------

/// Landlock filesystem access rights (bitflags-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LandlockFsAccess {
    Execute,
    WriteFile,
    ReadFile,
    ReadDir,
    RemoveDir,
    RemoveFile,
    MakeChar,
    MakeDir,
    MakeReg,
    MakeSock,
    MakeFifo,
    MakeBlock,
    MakeSym,
    Refer,
    Truncate,
}

/// A single Landlock filesystem rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandlockFsRule {
    pub path: std::path::PathBuf,
    pub allowed_access: Vec<LandlockFsAccess>,
}

/// Landlock network access rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LandlockNetAccess {
    BindTcp,
    ConnectTcp,
}

/// A single Landlock network rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandlockNetRule {
    pub port: u16,
    pub allowed_access: Vec<LandlockNetAccess>,
}

/// Complete Landlock ruleset for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LandlockRuleset {
    #[serde(default)]
    pub fs_rules: Vec<LandlockFsRule>,
    #[serde(default)]
    pub net_rules: Vec<LandlockNetRule>,
}

// ---------------------------------------------------------------------------
// Linux capabilities (POSIX 1003.1e)
// ---------------------------------------------------------------------------

/// Linux capability (POSIX 1003.1e).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LinuxCapability {
    CapChown,
    CapDacOverride,
    CapDacReadSearch,
    CapFowner,
    CapFsetid,
    CapKill,
    CapSetgid,
    CapSetuid,
    CapSetpcap,
    CapLinuxImmutable,
    CapNetBindService,
    CapNetBroadcast,
    CapNetAdmin,
    CapNetRaw,
    CapIpcLock,
    CapIpcOwner,
    CapSysModule,
    CapSysRawio,
    CapSysChroot,
    CapSysPtrace,
    CapSysPacketSocket,
    CapSysAdmin,
    CapSysBoot,
    CapSysNice,
    CapSysResource,
    CapSysTime,
    CapSysTtyConfig,
    CapMknod,
    CapLease,
    CapAuditWrite,
    CapAuditRead,
    CapAuditControl,
    CapSetfcap,
    CapSyslog,
    CapWakeAlarm,
    CapBlockSuspend,
    CapBpf,
    CapCheckpointRestore,
    CapPerfmon,
}

/// Linux capability sets for a sandboxed process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    #[serde(default)]
    pub effective: Vec<LinuxCapability>,
    #[serde(default)]
    pub permitted: Vec<LinuxCapability>,
    #[serde(default)]
    pub inheritable: Vec<LinuxCapability>,
    #[serde(default)]
    pub bounding: Vec<LinuxCapability>,
    #[serde(default)]
    pub ambient: Vec<LinuxCapability>,
}

// ---------------------------------------------------------------------------
// Sandbox capability detection
// ---------------------------------------------------------------------------

/// Seccomp BPF mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeccompMode {
    Disabled,
    Strict,
    Filter,
    Unsupported,
}

/// Detected sandbox capabilities on the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxCapabilities {
    pub seccomp_available: bool,
    pub seccomp_mode: SeccompMode,
    pub landlock_available: bool,
    /// Landlock ABI version (0 = not available).
    pub landlock_abi: u32,
    pub cgroup_v2: bool,
    pub namespaces_available: bool,
}

impl Default for SandboxCapabilities {
    fn default() -> Self {
        Self {
            seccomp_available: false,
            seccomp_mode: SeccompMode::Disabled,
            landlock_available: false,
            landlock_abi: 0,
            cgroup_v2: false,
            namespaces_available: false,
        }
    }
}

// ---------------------------------------------------------------------------
// RBAC
// ---------------------------------------------------------------------------

/// System role for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Role {
    Admin,
    Operator,
    Auditor,
    Viewer,
    Service,
}

/// Comparison operator for permission conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConditionOperator {
    Eq,
    Neq,
    In,
    Nin,
    Gt,
    Gte,
    Lt,
    Lte,
}

/// A condition on a permission rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

/// A resource-level permission with actions and optional conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    pub resource: String,
    pub actions: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<PermissionCondition>,
}

/// JWT-style token payload for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    /// Subject (user or agent ID).
    pub sub: String,
    pub role: Role,
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Issued-at timestamp (Unix seconds).
    pub iat: u64,
    /// Expiry timestamp (Unix seconds).
    pub exp: u64,
    /// Token ID (for revocation).
    pub jti: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Authentication context for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub agent_id: crate::types::AgentId,
    pub role: Role,
    pub permissions: Vec<RolePermission>,
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
            SeccompAction::Kill,
            SeccompAction::KillProcess,
            SeccompAction::Trap,
            SeccompAction::Errno(1),
            SeccompAction::Trace(0),
            SeccompAction::Log,
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
            names: vec!["write".into(), "writev".into()],
            action: SeccompAction::Allow,
            args: vec![SeccompArg {
                index: 0,
                value: 1,
                value_two: 0,
                op: SeccompArgOp::Equal,
            }],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: SeccompRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.names, vec!["write", "writev"]);
        assert_eq!(back.action, SeccompAction::Allow);
        assert_eq!(back.args.len(), 1);
        assert_eq!(back.args[0].op, SeccompArgOp::Equal);
    }

    #[test]
    fn seccomp_profile_serde_roundtrip() {
        let p = SeccompProfile {
            default_action: SeccompAction::Errno(1),
            architectures: vec![SeccompArch::X86_64, SeccompArch::Aarch64],
            flags: vec!["SECCOMP_FILTER_FLAG_LOG".into()],
            syscalls: vec![SeccompRule {
                names: vec!["read".into(), "write".into(), "exit_group".into()],
                action: SeccompAction::Allow,
                args: vec![],
            }],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: SeccompProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.default_action, SeccompAction::Errno(1));
        assert_eq!(back.architectures.len(), 2);
        assert_eq!(back.syscalls.len(), 1);
    }

    #[test]
    fn seccomp_arch_serde_roundtrip() {
        for variant in [
            SeccompArch::X86,
            SeccompArch::X86_64,
            SeccompArch::Aarch64,
            SeccompArch::Riscv64,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SeccompArch = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn seccomp_arg_op_serde_roundtrip() {
        for variant in [
            SeccompArgOp::NotEqual,
            SeccompArgOp::LessThan,
            SeccompArgOp::Equal,
            SeccompArgOp::GreaterThan,
            SeccompArgOp::MaskedEqual,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SeccompArgOp = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
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
        let id = AgentId::new();
        let ctx = SecurityContext {
            agent_id: id,
            permissions: vec![Permission::FileRead, Permission::NetworkAccess],
            sandbox: SandboxConfig::default(),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let back: SecurityContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, id);
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

    #[test]
    fn system_feature_is_capability_alias() {
        let sf = SystemFeature::Landlock;
        let cap: Capability = sf;
        assert_eq!(sf, cap);
    }

    // --- Cgroup tests ---

    #[test]
    fn cgroup_limits_default() {
        let c = CgroupLimits::default();
        assert!(c.memory_max.is_none());
        assert!(c.pids_max.is_none());
    }

    #[test]
    fn cgroup_limits_serde_roundtrip() {
        let c = CgroupLimits {
            memory_max: Some(1024 * 1024 * 512),
            memory_high: Some(1024 * 1024 * 256),
            cpu_max_usec: Some(50000),
            cpu_period_usec: Some(100000),
            cpu_weight: Some(100),
            pids_max: Some(64),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: CgroupLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    // --- Namespace tests ---

    #[test]
    fn namespace_config_default() {
        let n = NamespaceConfig::default();
        assert!(!n.pid);
        assert!(!n.net);
    }

    #[test]
    fn namespace_config_serde_roundtrip() {
        let n = NamespaceConfig {
            pid: true,
            net: true,
            mount: true,
            user: true,
            uts: false,
            ipc: false,
            cgroup: true,
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: NamespaceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(n, back);
    }

    #[test]
    fn id_mapping_serde_roundtrip() {
        let m = IdMapping {
            inside_id: 0,
            outside_id: 1000,
            count: 1,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: IdMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    // --- Landlock tests ---

    #[test]
    fn landlock_fs_access_serde_roundtrip() {
        for variant in [
            LandlockFsAccess::Execute,
            LandlockFsAccess::ReadFile,
            LandlockFsAccess::WriteFile,
            LandlockFsAccess::ReadDir,
            LandlockFsAccess::MakeDir,
            LandlockFsAccess::Truncate,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LandlockFsAccess = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn landlock_fs_rule_serde_roundtrip() {
        let r = LandlockFsRule {
            path: "/home/user".into(),
            allowed_access: vec![LandlockFsAccess::ReadFile, LandlockFsAccess::ReadDir],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: LandlockFsRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, std::path::PathBuf::from("/home/user"));
        assert_eq!(back.allowed_access.len(), 2);
    }

    #[test]
    fn landlock_net_access_serde_roundtrip() {
        for variant in [LandlockNetAccess::BindTcp, LandlockNetAccess::ConnectTcp] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LandlockNetAccess = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn landlock_ruleset_serde_roundtrip() {
        let rs = LandlockRuleset {
            fs_rules: vec![LandlockFsRule {
                path: "/tmp".into(),
                allowed_access: vec![LandlockFsAccess::ReadFile, LandlockFsAccess::WriteFile],
            }],
            net_rules: vec![LandlockNetRule {
                port: 443,
                allowed_access: vec![LandlockNetAccess::ConnectTcp],
            }],
        };
        let json = serde_json::to_string(&rs).unwrap();
        let back: LandlockRuleset = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fs_rules.len(), 1);
        assert_eq!(back.net_rules.len(), 1);
    }

    // --- Linux capability tests ---

    #[test]
    fn linux_capability_serde_roundtrip() {
        for variant in [
            LinuxCapability::CapNetAdmin,
            LinuxCapability::CapSysAdmin,
            LinuxCapability::CapNetRaw,
            LinuxCapability::CapSetuid,
            LinuxCapability::CapSysChroot,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LinuxCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn capability_set_default() {
        let cs = CapabilitySet::default();
        assert!(cs.effective.is_empty());
        assert!(cs.bounding.is_empty());
    }

    #[test]
    fn capability_set_serde_roundtrip() {
        let cs = CapabilitySet {
            effective: vec![LinuxCapability::CapNetBindService],
            permitted: vec![
                LinuxCapability::CapNetBindService,
                LinuxCapability::CapNetRaw,
            ],
            inheritable: vec![],
            bounding: vec![LinuxCapability::CapNetBindService],
            ambient: vec![],
        };
        let json = serde_json::to_string(&cs).unwrap();
        let back: CapabilitySet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.effective.len(), 1);
        assert_eq!(back.permitted.len(), 2);
    }

    // --- Sandbox capabilities tests ---

    #[test]
    fn seccomp_mode_serde_roundtrip() {
        for variant in [
            SeccompMode::Disabled,
            SeccompMode::Strict,
            SeccompMode::Filter,
            SeccompMode::Unsupported,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SeccompMode = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn sandbox_capabilities_default() {
        let sc = SandboxCapabilities::default();
        assert!(!sc.seccomp_available);
        assert_eq!(sc.seccomp_mode, SeccompMode::Disabled);
        assert!(!sc.landlock_available);
        assert_eq!(sc.landlock_abi, 0);
    }

    #[test]
    fn sandbox_capabilities_serde_roundtrip() {
        let sc = SandboxCapabilities {
            seccomp_available: true,
            seccomp_mode: SeccompMode::Filter,
            landlock_available: true,
            landlock_abi: 4,
            cgroup_v2: true,
            namespaces_available: true,
        };
        let json = serde_json::to_string(&sc).unwrap();
        let back: SandboxCapabilities = serde_json::from_str(&json).unwrap();
        assert!(back.seccomp_available);
        assert_eq!(back.seccomp_mode, SeccompMode::Filter);
        assert_eq!(back.landlock_abi, 4);
    }

    // --- RBAC tests ---

    #[test]
    fn role_serde_roundtrip() {
        for variant in [
            Role::Admin,
            Role::Operator,
            Role::Auditor,
            Role::Viewer,
            Role::Service,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: Role = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn condition_operator_serde_roundtrip() {
        for variant in [
            ConditionOperator::Eq,
            ConditionOperator::Neq,
            ConditionOperator::In,
            ConditionOperator::Gt,
            ConditionOperator::Lte,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ConditionOperator = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn role_permission_serde_roundtrip() {
        let rp = RolePermission {
            resource: "agents".into(),
            actions: vec!["read".into(), "write".into()],
            conditions: vec![PermissionCondition {
                field: "owner".into(),
                operator: ConditionOperator::Eq,
                value: serde_json::json!("self"),
            }],
        };
        let json = serde_json::to_string(&rp).unwrap();
        let back: RolePermission = serde_json::from_str(&json).unwrap();
        assert_eq!(back.resource, "agents");
        assert_eq!(back.actions.len(), 2);
        assert_eq!(back.conditions.len(), 1);
    }

    #[test]
    fn token_payload_serde_roundtrip() {
        let tp = TokenPayload {
            sub: "user-001".into(),
            role: Role::Operator,
            permissions: vec!["agents:read".into(), "agents:write".into()],
            iat: 1711324800,
            exp: 1711411200,
            jti: "tok-abc-123".into(),
            email: Some("user@example.com".into()),
            display_name: Some("Test User".into()),
        };
        let json = serde_json::to_string(&tp).unwrap();
        let back: TokenPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sub, "user-001");
        assert_eq!(back.role, Role::Operator);
        assert_eq!(back.permissions.len(), 2);
        assert_eq!(back.email.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn auth_context_serde_roundtrip() {
        let ac = AuthContext {
            agent_id: AgentId::new(),
            role: Role::Admin,
            permissions: vec![RolePermission {
                resource: "*".into(),
                actions: vec!["*".into()],
                conditions: vec![],
            }],
        };
        let json = serde_json::to_string(&ac).unwrap();
        let back: AuthContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.role, Role::Admin);
        assert_eq!(back.permissions.len(), 1);
    }
}
