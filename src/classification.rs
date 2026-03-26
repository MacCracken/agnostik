use serde::{Deserialize, Serialize};

/// Data classification level (ordered by sensitivity).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[non_exhaustive]
pub enum ClassificationLevel {
    Public,
    #[default]
    Internal,
    Confidential,
    Restricted,
}

/// Kind of personally identifiable information detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PiiKind {
    Email,
    Phone,
    Ssn,
    CreditCard,
    IpAddress,
    Passport,
    DriversLicense,
    DateOfBirth,
    Custom,
}

/// Result of classifying content for data sensitivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Final classification level (may be overridden by policy).
    pub level: ClassificationLevel,
    /// Level determined by automatic detection rules.
    pub auto_level: ClassificationLevel,
    /// Names of rules that triggered.
    #[serde(default)]
    pub rules_triggered: Vec<String>,
    /// PII types found in the content.
    #[serde(default)]
    pub pii_found: Vec<PiiKind>,
    /// Sensitive keywords found.
    #[serde(default)]
    pub keywords_found: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_level_ordering() {
        assert!(ClassificationLevel::Public < ClassificationLevel::Internal);
        assert!(ClassificationLevel::Internal < ClassificationLevel::Confidential);
        assert!(ClassificationLevel::Confidential < ClassificationLevel::Restricted);
    }

    #[test]
    fn classification_level_default() {
        assert_eq!(
            ClassificationLevel::default(),
            ClassificationLevel::Internal
        );
    }

    #[test]
    fn classification_level_serde_roundtrip() {
        for variant in [
            ClassificationLevel::Public,
            ClassificationLevel::Internal,
            ClassificationLevel::Confidential,
            ClassificationLevel::Restricted,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ClassificationLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn pii_kind_serde_roundtrip() {
        for variant in [
            PiiKind::Email,
            PiiKind::Phone,
            PiiKind::Ssn,
            PiiKind::CreditCard,
            PiiKind::IpAddress,
            PiiKind::Passport,
            PiiKind::DriversLicense,
            PiiKind::DateOfBirth,
            PiiKind::Custom,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: PiiKind = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn classification_result_serde_roundtrip() {
        let r = ClassificationResult {
            level: ClassificationLevel::Confidential,
            auto_level: ClassificationLevel::Confidential,
            rules_triggered: vec!["pii_detected".into()],
            pii_found: vec![PiiKind::Email, PiiKind::Phone],
            keywords_found: vec!["secret".into()],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ClassificationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.level, ClassificationLevel::Confidential);
        assert_eq!(back.pii_found.len(), 2);
        assert_eq!(back.keywords_found, vec!["secret"]);
    }
}
