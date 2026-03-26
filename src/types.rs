use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    #[serde(default)]
    pub prerelease: Option<String>,
    #[serde(default)]
    pub build: Option<String>,
}

impl Default for Version {
    fn default() -> Self {
        Self { major: 2026, minor: 3, patch: 25, prerelease: None, build: None }
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

/// System capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_true() -> bool { true }

impl Default for Capabilities {
    fn default() -> Self {
        Self { llm_support: false, virtualization: true, gpu_available: false, tpm_available: false }
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
        let v = Version { major: 1, minor: 2, patch: 3, prerelease: Some("alpha".into()), build: Some("b1".into()) };
        assert_eq!(v.to_string(), "1.2.3-alpha+b1");
    }

    #[test]
    fn version_display_simple() {
        let v = Version { major: 2, minor: 0, patch: 0, prerelease: None, build: None };
        assert_eq!(v.to_string(), "2.0.0");
    }

    #[test]
    fn version_default() {
        let v = Version::default();
        assert_eq!(v.major, 2026);
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
        let c = ComponentConfig { name: "test".into(), enabled: true, settings };
        let json = serde_json::to_string(&c).unwrap();
        let back: ComponentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
    }

    #[test]
    fn version_serde_roundtrip() {
        let v = Version { major: 1, minor: 0, patch: 0, prerelease: None, build: None };
        let json = serde_json::to_string(&v).unwrap();
        let back: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}
