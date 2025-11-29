//! Oracle configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the decy oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OracleConfig {
    /// Path to the patterns file (.apr format)
    pub patterns_path: PathBuf,

    /// Confidence threshold for suggestions (0.0-1.0)
    pub confidence_threshold: f32,

    /// Maximum suggestions to return
    pub max_suggestions: usize,

    /// Enable auto-fix for high-confidence suggestions
    pub auto_fix: bool,

    /// Maximum retry attempts with oracle fixes
    pub max_retries: usize,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            patterns_path: default_patterns_path(),
            confidence_threshold: 0.7_f32,
            max_suggestions: 5,
            auto_fix: false,
            max_retries: 3,
        }
    }
}

fn default_patterns_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".decy")
        .join("decision_patterns.apr")
}

impl OracleConfig {
    /// Create config from environment variables
    ///
    /// Looks for:
    /// - DECY_ORACLE_PATTERNS: Path to patterns file
    /// - DECY_ORACLE_THRESHOLD: Confidence threshold
    /// - DECY_ORACLE_AUTO_FIX: Enable auto-fix (true/false)
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(path) = std::env::var("DECY_ORACLE_PATTERNS") {
            config.patterns_path = PathBuf::from(path);
        }

        if let Ok(threshold) = std::env::var("DECY_ORACLE_THRESHOLD") {
            if let Ok(t) = threshold.parse() {
                config.confidence_threshold = t;
            }
        }

        if let Ok(auto_fix) = std::env::var("DECY_ORACLE_AUTO_FIX") {
            config.auto_fix = auto_fix.to_lowercase() == "true";
        }

        config
    }

    /// Load config from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, toml::de::Error> {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = OracleConfig::default();
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 5);
        assert!(!config.auto_fix);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
confidence_threshold = 0.85
max_suggestions = 10
auto_fix = true
max_retries = 5
"#;
        let config: OracleConfig = toml::from_str(toml).unwrap();
        assert!((config.confidence_threshold - 0.85_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 10);
        assert!(config.auto_fix);
        assert_eq!(config.max_retries, 5);
    }
}
