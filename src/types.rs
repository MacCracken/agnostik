use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique agent identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AgentId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::str::FromStr for AgentId {
    type Err = crate::error::AgnostikError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self).map_err(|e| {
            crate::error::AgnostikError::InvalidArgument(format!(
                "invalid agent id: {s} (expected UUID, {e})"
            ))
        })
    }
}

/// Unique user identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for UserId {
    type Err = crate::error::AgnostikError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self).map_err(|e| {
            crate::error::AgnostikError::InvalidArgument(format!(
                "invalid user id: {s} (expected UUID, {e})"
            ))
        })
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Version information (SemVer).
///
/// Serializes as a string `"MAJOR.MINOR.PATCH[-PRE][+BUILD]"`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.prerelease {
            write!(f, "-{pre}")?;
        }
        if let Some(build) = &self.build {
            write!(f, "+{build}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Version {
    type Err = crate::error::AgnostikError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // Split off build metadata first (after '+')
        let (rest, build) = match s.split_once('+') {
            Some((r, b)) => (r, Some(b.to_owned())),
            None => (s, None),
        };
        // Split off prerelease (after '-', but only after the version core)
        let (core, prerelease) = match rest.split_once('-') {
            Some((c, p)) => (c, Some(p.to_owned())),
            None => (rest, None),
        };
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::AgnostikError::InvalidArgument(format!(
                "invalid version: {s} (expected MAJOR.MINOR.PATCH)"
            )));
        }
        let parse = |p: &str| {
            p.parse::<u32>().map_err(|_| {
                crate::error::AgnostikError::InvalidArgument(format!(
                    "invalid version component: {p} (expected unsigned integer)"
                ))
            })
        };
        Ok(Self {
            major: parse(parts[0])?,
            minor: parse(parts[1])?,
            patch: parse(parts[2])?,
            prerelease,
            build,
        })
    }
}

/// System capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub llm_support: bool,
    #[serde(default = "default_true")]
    pub virtualization: bool,
    #[serde(default)]
    pub gpu_available: bool,
    #[serde(default)]
    pub tpm_available: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            llm_support: false,
            virtualization: true,
            gpu_available: false,
            tpm_available: false,
        }
    }
}

/// IPC message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MessageType {
    Command,
    Response,
    Event,
    Heartbeat,
}

/// System health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Component configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub name: String,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_display() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: Some("alpha".into()),
            build: Some("b1".into()),
        };
        assert_eq!(v.to_string(), "1.2.3-alpha+b1");
    }

    #[test]
    fn version_display_simple() {
        let v = Version {
            major: 2,
            minor: 0,
            patch: 0,
            prerelease: None,
            build: None,
        };
        assert_eq!(v.to_string(), "2.0.0");
    }

    #[test]
    fn version_default() {
        let v = Version::default();
        assert_eq!(v.to_string(), "0.0.0");
    }

    #[test]
    fn capabilities_default() {
        let c = Capabilities::default();
        assert!(!c.llm_support);
        assert!(c.virtualization);
    }

    #[test]
    fn message_type_eq() {
        assert_eq!(MessageType::Command, MessageType::Command);
        assert_ne!(MessageType::Command, MessageType::Event);
    }

    #[test]
    fn system_status_eq() {
        assert_eq!(SystemStatus::Healthy, SystemStatus::Healthy);
        assert_ne!(SystemStatus::Healthy, SystemStatus::Critical);
    }

    #[test]
    fn component_config_serde() {
        let mut settings = HashMap::new();
        settings.insert("port".into(), serde_json::json!(8080));
        let c = ComponentConfig {
            name: "test".into(),
            enabled: true,
            settings,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: ComponentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
    }

    #[test]
    fn version_serde_roundtrip() {
        let v = Version {
            major: 1,
            minor: 0,
            patch: 0,
            prerelease: None,
            build: None,
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn capabilities_serde_roundtrip() {
        let c = Capabilities::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: Capabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(back.virtualization, c.virtualization);
        assert_eq!(back.llm_support, c.llm_support);
        assert_eq!(back.gpu_available, c.gpu_available);
        assert_eq!(back.tpm_available, c.tpm_available);
    }

    #[test]
    fn message_type_serde_roundtrip() {
        for variant in [
            MessageType::Command,
            MessageType::Response,
            MessageType::Event,
            MessageType::Heartbeat,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: MessageType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn system_status_serde_roundtrip() {
        for variant in [
            SystemStatus::Healthy,
            SystemStatus::Degraded,
            SystemStatus::Critical,
            SystemStatus::Unknown,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SystemStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn agent_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id: AgentId = uuid.into();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn agent_id_from_str_roundtrip() {
        let id = AgentId::new();
        let s = id.to_string();
        let parsed: AgentId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn agent_id_from_str_invalid() {
        assert!("not-a-uuid".parse::<AgentId>().is_err());
    }

    #[test]
    fn user_id_serde_roundtrip() {
        let id = UserId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn version_from_str_simple() {
        let v: Version = "1.2.3".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.prerelease.is_none());
        assert!(v.build.is_none());
    }

    #[test]
    fn version_from_str_full() {
        let v: Version = "1.2.3-alpha+b1".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.prerelease.as_deref(), Some("alpha"));
        assert_eq!(v.build.as_deref(), Some("b1"));
    }

    #[test]
    fn version_from_str_display_roundtrip() {
        let original = "1.2.3-beta.1+build.42";
        let v: Version = original.parse().unwrap();
        assert_eq!(v.to_string(), original);
    }

    #[test]
    fn version_from_str_invalid() {
        assert!("1.2".parse::<Version>().is_err());
        assert!("abc".parse::<Version>().is_err());
    }

    #[test]
    fn version_from_str_invalid_part() {
        assert!("1.2.xyz".parse::<Version>().is_err());
    }

    #[test]
    fn agent_id_default() {
        let a = AgentId::default();
        let b = AgentId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn user_id_default() {
        let a = UserId::default();
        let b = UserId::default();
        assert_ne!(a, b);
    }

    #[test]
    fn user_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id: UserId = uuid.into();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn user_id_display() {
        let id = UserId::new();
        let s = format!("{id}");
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn user_id_from_str_roundtrip() {
        let id = UserId::new();
        let s = id.to_string();
        let parsed: UserId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn user_id_from_str_invalid() {
        assert!("not-a-uuid".parse::<UserId>().is_err());
    }

    #[test]
    fn version_serde_as_string() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: Some("alpha".into()),
            build: None,
        };
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#""1.2.3-alpha""#);
        let back: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn capabilities_eq() {
        let a = Capabilities::default();
        let b = Capabilities::default();
        assert_eq!(a, b);
    }

    #[test]
    fn capabilities_serde_with_defaults() {
        // Deserialize with missing fields to exercise serde defaults
        let json = r#"{"llm_support":true}"#;
        let c: Capabilities = serde_json::from_str(json).unwrap();
        assert!(c.llm_support);
        assert!(c.virtualization); // default_true()
        assert!(!c.gpu_available);
    }
}
