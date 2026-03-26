use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment profile for agent deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EnvironmentProfile {
    Development,
    Staging,
    Production,
    Edge,
}

impl Default for EnvironmentProfile {
    fn default() -> Self { Self::Development }
}

/// Top-level AGNOS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgnosConfig {
    pub profile: EnvironmentProfile,
    pub hostname: String,
    #[serde(default)]
    pub components: HashMap<String, crate::types::ComponentConfig>,
}

impl Default for AgnosConfig {
    fn default() -> Self {
        Self {
            profile: EnvironmentProfile::Development,
            hostname: "localhost".into(),
            components: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_default() {
        assert_eq!(EnvironmentProfile::default(), EnvironmentProfile::Development);
    }

    #[test]
    fn config_default() {
        let c = AgnosConfig::default();
        assert_eq!(c.hostname, "localhost");
        assert!(c.components.is_empty());
    }

    #[test]
    fn config_serde() {
        let c = AgnosConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: AgnosConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.profile, EnvironmentProfile::Development);
    }
}
