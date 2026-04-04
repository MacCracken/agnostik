use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment profile for agent deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum EnvironmentProfile {
    #[default]
    Development,
    Testing,
    Staging,
    Canary,
    Production,
    Edge,
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

/// Edge-specific resource overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeResourceOverrides {
    /// Override max memory (bytes).
    #[serde(default)]
    pub max_memory: Option<u64>,
    /// Override max CPU time (milliseconds).
    #[serde(default)]
    pub max_cpu_time: Option<u64>,
    /// Override max concurrent agents.
    #[serde(default)]
    pub max_agents: Option<u32>,
}

/// A profile definition with optional inheritance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDefinition {
    pub name: String,
    pub profile: EnvironmentProfile,
    /// Parent profile to inherit settings from.
    #[serde(default)]
    pub inherits_from: Option<EnvironmentProfile>,
    /// Edge-specific resource overrides (applied on top of inherited settings).
    #[serde(default)]
    pub edge_overrides: Option<EdgeResourceOverrides>,
    /// Profile-specific settings (override inherited component settings).
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Fleet-wide configuration for distributing profiles to nodes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FleetConfig {
    /// Named profiles available in the fleet.
    #[serde(default)]
    pub profiles: Vec<ProfileDefinition>,
    /// Default profile assigned to new nodes.
    #[serde(default)]
    pub default_profile: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_default() {
        assert_eq!(
            EnvironmentProfile::default(),
            EnvironmentProfile::Development
        );
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

    #[test]
    fn environment_profile_serde_roundtrip() {
        for variant in [
            EnvironmentProfile::Development,
            EnvironmentProfile::Staging,
            EnvironmentProfile::Production,
            EnvironmentProfile::Edge,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EnvironmentProfile = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn edge_resource_overrides_default() {
        let e = EdgeResourceOverrides::default();
        assert!(e.max_memory.is_none());
        assert!(e.max_agents.is_none());
    }

    #[test]
    fn profile_definition_serde_roundtrip() {
        let pd = ProfileDefinition {
            name: "edge-low-power".into(),
            profile: EnvironmentProfile::Edge,
            inherits_from: Some(EnvironmentProfile::Production),
            edge_overrides: Some(EdgeResourceOverrides {
                max_memory: Some(512 * 1024 * 1024),
                max_cpu_time: None,
                max_agents: Some(4),
            }),
            settings: [("log_level".into(), serde_json::json!("warn"))]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&pd).unwrap();
        let back: ProfileDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "edge-low-power");
        assert_eq!(back.inherits_from, Some(EnvironmentProfile::Production));
        assert!(back.edge_overrides.is_some());
    }

    #[test]
    fn edge_resource_overrides_serde_roundtrip() {
        let e = EdgeResourceOverrides {
            max_memory: Some(1024 * 1024 * 1024),
            max_cpu_time: Some(5000),
            max_agents: Some(8),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: EdgeResourceOverrides = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_memory, Some(1024 * 1024 * 1024));
        assert_eq!(back.max_cpu_time, Some(5000));
        assert_eq!(back.max_agents, Some(8));
    }

    #[test]
    fn fleet_config_serde_roundtrip() {
        let fc = FleetConfig {
            profiles: vec![ProfileDefinition {
                name: "default".into(),
                profile: EnvironmentProfile::Production,
                inherits_from: None,
                edge_overrides: None,
                settings: HashMap::new(),
            }],
            default_profile: Some("default".into()),
        };
        let json = serde_json::to_string(&fc).unwrap();
        let back: FleetConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.profiles.len(), 1);
        assert_eq!(back.default_profile.as_deref(), Some("default"));
    }
}
