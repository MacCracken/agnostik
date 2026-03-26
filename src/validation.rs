use serde::{Deserialize, Serialize};

/// Severity of a validation warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
}

/// A warning generated during input validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Machine-readable warning code (e.g. "xss_detected", "sql_injection").
    pub code: String,
    /// Human-readable description.
    pub message: String,
    pub severity: ValidationSeverity,
    /// Character position where the issue was detected.
    #[serde(default)]
    pub position: Option<usize>,
    /// Pattern or snippet that triggered the warning.
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Result of validating and sanitizing user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the input passed validation.
    pub valid: bool,
    /// Sanitized version of the input.
    pub sanitized: String,
    /// Warnings generated during validation.
    #[serde(default)]
    pub warnings: Vec<ValidationWarning>,
    /// Whether the input was blocked entirely.
    #[serde(default)]
    pub blocked: bool,
    /// Reason for blocking (if blocked).
    #[serde(default)]
    pub block_reason: Option<String>,
    /// Injection threat score (0.0 = safe, 1.0 = highly suspicious).
    #[serde(default)]
    pub injection_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_severity_ordering() {
        assert!(ValidationSeverity::Low < ValidationSeverity::Medium);
        assert!(ValidationSeverity::Medium < ValidationSeverity::High);
    }

    #[test]
    fn validation_severity_serde_roundtrip() {
        for variant in [
            ValidationSeverity::Low,
            ValidationSeverity::Medium,
            ValidationSeverity::High,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ValidationSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn validation_warning_serde_roundtrip() {
        let w = ValidationWarning {
            code: "xss_detected".into(),
            message: "Potential XSS in input".into(),
            severity: ValidationSeverity::High,
            position: Some(42),
            pattern: Some("<script>".into()),
        };
        let json = serde_json::to_string(&w).unwrap();
        let back: ValidationWarning = serde_json::from_str(&json).unwrap();
        assert_eq!(back.code, "xss_detected");
        assert_eq!(back.severity, ValidationSeverity::High);
        assert_eq!(back.position, Some(42));
    }

    #[test]
    fn validation_result_serde_roundtrip() {
        let r = ValidationResult {
            valid: false,
            sanitized: "cleaned input".into(),
            warnings: vec![ValidationWarning {
                code: "sql_injection".into(),
                message: "SQL injection attempt".into(),
                severity: ValidationSeverity::High,
                position: None,
                pattern: Some("' OR 1=1".into()),
            }],
            blocked: true,
            block_reason: Some("injection detected".into()),
            injection_score: 0.95,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ValidationResult = serde_json::from_str(&json).unwrap();
        assert!(!back.valid);
        assert!(back.blocked);
        assert_eq!(back.warnings.len(), 1);
        assert!((back.injection_score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn validation_result_clean_input() {
        let r = ValidationResult {
            valid: true,
            sanitized: "hello world".into(),
            warnings: vec![],
            blocked: false,
            block_reason: None,
            injection_score: 0.0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ValidationResult = serde_json::from_str(&json).unwrap();
        assert!(back.valid);
        assert!(!back.blocked);
        assert!(back.warnings.is_empty());
    }
}
