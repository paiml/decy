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
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ============================================================================
    // DEFAULT CONFIGURATION TESTS
    // ============================================================================

    #[test]
    fn test_config_default() {
        let config = OracleConfig::default();
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 5);
        assert!(!config.auto_fix);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_default_patterns_path_ends_with_apr() {
        let config = OracleConfig::default();
        assert!(config.patterns_path.to_string_lossy().ends_with(".apr"));
    }

    #[test]
    fn test_config_default_patterns_path_in_decy_dir() {
        let config = OracleConfig::default();
        assert!(config.patterns_path.to_string_lossy().contains(".decy"));
    }

    // ============================================================================
    // TOML SERIALIZATION/DESERIALIZATION TESTS
    // ============================================================================

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

    #[test]
    fn test_config_from_toml_partial() {
        // Test that missing fields use defaults (via #[serde(default)])
        let toml = r#"
confidence_threshold = 0.9
"#;
        let config: OracleConfig = toml::from_str(toml).unwrap();
        assert!((config.confidence_threshold - 0.9_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 5); // default
        assert!(!config.auto_fix); // default
    }

    #[test]
    fn test_config_from_toml_empty() {
        // Empty TOML should use all defaults
        let toml = "";
        let config: OracleConfig = toml::from_str(toml).unwrap();
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 5);
    }

    #[test]
    fn test_config_to_toml() {
        let config = OracleConfig {
            patterns_path: PathBuf::from("/custom/path.apr"),
            confidence_threshold: 0.95,
            max_suggestions: 20,
            auto_fix: true,
            max_retries: 10,
        };
        let toml_str = toml::to_string(&config).unwrap();
        // Float may be serialized with varying precision, check for the key
        assert!(toml_str.contains("confidence_threshold"));
        assert!(toml_str.contains("max_suggestions = 20"));
        assert!(toml_str.contains("auto_fix = true"));
        assert!(toml_str.contains("max_retries = 10"));
        // Verify we can deserialize back
        let deserialized: OracleConfig = toml::from_str(&toml_str).unwrap();
        assert!((deserialized.confidence_threshold - 0.95).abs() < f32::EPSILON);
    }

    // ============================================================================
    // FROM_ENV TESTS
    // ============================================================================

    #[test]
    fn test_from_env_default_when_no_env_vars() {
        // Clear relevant env vars if set
        std::env::remove_var("DECY_ORACLE_PATTERNS");
        std::env::remove_var("DECY_ORACLE_THRESHOLD");
        std::env::remove_var("DECY_ORACLE_AUTO_FIX");

        let config = OracleConfig::from_env();
        // Should use defaults when no env vars set
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        assert!(!config.auto_fix);
    }

    #[test]
    fn test_from_env_patterns_path() {
        std::env::set_var("DECY_ORACLE_PATTERNS", "/custom/test/path.apr");
        let config = OracleConfig::from_env();
        assert_eq!(config.patterns_path, PathBuf::from("/custom/test/path.apr"));
        std::env::remove_var("DECY_ORACLE_PATTERNS");
    }

    #[test]
    fn test_from_env_threshold() {
        std::env::set_var("DECY_ORACLE_THRESHOLD", "0.85");
        let config = OracleConfig::from_env();
        assert!((config.confidence_threshold - 0.85_f32).abs() < f32::EPSILON);
        std::env::remove_var("DECY_ORACLE_THRESHOLD");
    }

    #[test]
    fn test_from_env_threshold_invalid_uses_default() {
        std::env::set_var("DECY_ORACLE_THRESHOLD", "not_a_number");
        let config = OracleConfig::from_env();
        // Should use default when parse fails
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        std::env::remove_var("DECY_ORACLE_THRESHOLD");
    }

    // Note: These tests are flaky when run in parallel due to env var races.
    // Run with: cargo test -p decy-oracle -- --test-threads=1
    #[test]
    #[ignore = "flaky: env var race condition, run with --test-threads=1"]
    fn test_from_env_auto_fix_true() {
        std::env::set_var("DECY_ORACLE_AUTO_FIX", "true");
        let config = OracleConfig::from_env();
        assert!(config.auto_fix);
        std::env::remove_var("DECY_ORACLE_AUTO_FIX");
    }

    #[test]
    #[ignore = "flaky: env var race condition, run with --test-threads=1"]
    fn test_from_env_auto_fix_true_uppercase() {
        std::env::set_var("DECY_ORACLE_AUTO_FIX", "TRUE");
        let config = OracleConfig::from_env();
        assert!(config.auto_fix);
        std::env::remove_var("DECY_ORACLE_AUTO_FIX");
    }

    #[test]
    #[ignore = "flaky: env var race condition, run with --test-threads=1"]
    fn test_from_env_auto_fix_false() {
        std::env::set_var("DECY_ORACLE_AUTO_FIX", "false");
        let config = OracleConfig::from_env();
        assert!(!config.auto_fix);
        std::env::remove_var("DECY_ORACLE_AUTO_FIX");
    }

    #[test]
    #[ignore = "flaky: env var race condition, run with --test-threads=1"]
    fn test_from_env_auto_fix_any_other_value_is_false() {
        std::env::set_var("DECY_ORACLE_AUTO_FIX", "yes");
        let config = OracleConfig::from_env();
        assert!(!config.auto_fix); // "yes" != "true"
        std::env::remove_var("DECY_ORACLE_AUTO_FIX");
    }

    // ============================================================================
    // FROM_FILE TESTS
    // ============================================================================

    #[test]
    fn test_from_file_valid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
confidence_threshold = 0.8
max_suggestions = 15
auto_fix = true
max_retries = 7
"#
        )
        .unwrap();

        let config = OracleConfig::from_file(file.path()).unwrap();
        assert!((config.confidence_threshold - 0.8_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 15);
        assert!(config.auto_fix);
        assert_eq!(config.max_retries, 7);
    }

    #[test]
    fn test_from_file_nonexistent_uses_defaults() {
        // When file doesn't exist, read_to_string returns "" via unwrap_or_default
        // Empty string parsed as TOML gives defaults
        let config = OracleConfig::from_file(std::path::Path::new("/nonexistent/path.toml"));
        // Should use defaults
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_from_file_empty_file_uses_defaults() {
        let file = NamedTempFile::new().unwrap();
        // Don't write anything - empty file

        let config = OracleConfig::from_file(file.path()).unwrap();
        assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, 5);
    }

    #[test]
    fn test_from_file_invalid_toml_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid = [toml").unwrap();

        let result = OracleConfig::from_file(file.path());
        assert!(result.is_err());
    }

    // ============================================================================
    // DEBUG AND CLONE TESTS
    // ============================================================================

    #[test]
    fn test_config_debug() {
        let config = OracleConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("OracleConfig"));
        assert!(debug_str.contains("confidence_threshold"));
    }

    #[test]
    fn test_config_clone() {
        let config = OracleConfig {
            patterns_path: PathBuf::from("/test/path.apr"),
            confidence_threshold: 0.9,
            max_suggestions: 10,
            auto_fix: true,
            max_retries: 5,
        };
        let cloned = config.clone();
        assert_eq!(config.patterns_path, cloned.patterns_path);
        assert!((config.confidence_threshold - cloned.confidence_threshold).abs() < f32::EPSILON);
        assert_eq!(config.max_suggestions, cloned.max_suggestions);
        assert_eq!(config.auto_fix, cloned.auto_fix);
    }

    // ============================================================================
    // DEFAULT_PATTERNS_PATH FUNCTION TESTS
    // ============================================================================

    #[test]
    fn test_default_patterns_path_returns_path() {
        let path = default_patterns_path();
        // Should be a non-empty path
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_default_patterns_path_file_extension() {
        let path = default_patterns_path();
        assert_eq!(path.extension().unwrap(), "apr");
    }

    #[test]
    fn test_default_patterns_path_filename() {
        let path = default_patterns_path();
        assert_eq!(path.file_name().unwrap(), "decision_patterns.apr");
    }
}
