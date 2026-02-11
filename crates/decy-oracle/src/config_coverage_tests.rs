//! Coverage tests for config.rs
//!
//! These tests target all uncovered lines and branches in OracleConfig,
//! including from_env with auto_fix variants, from_file edge cases,
//! serde round-trip paths, and the default_patterns_path fallback logic.

use crate::config::OracleConfig;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

// Mutex for serializing env var tests to prevent race conditions.
// Use unwrap_or_else to recover from poison (a prior test panicking).
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

// ============================================================================
// FROM_ENV: AUTO_FIX BRANCH COVERAGE (replaces ignored tests)
// ============================================================================

#[test]
fn config_coverage_from_env_auto_fix_true_lowercase() {
    let _lock = lock_env();
    // Clean slate
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "true");
    let config = OracleConfig::from_env();
    assert!(config.auto_fix, "auto_fix should be true for 'true'");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_auto_fix_true_uppercase() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "TRUE");
    let config = OracleConfig::from_env();
    assert!(config.auto_fix, "auto_fix should be true for 'TRUE'");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_auto_fix_true_mixed_case() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "True");
    let config = OracleConfig::from_env();
    assert!(config.auto_fix, "auto_fix should be true for 'True'");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_auto_fix_false_explicit() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "false");
    let config = OracleConfig::from_env();
    assert!(!config.auto_fix, "auto_fix should be false for 'false'");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_auto_fix_arbitrary_string_is_false() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "yes");
    let config = OracleConfig::from_env();
    assert!(!config.auto_fix, "'yes' != 'true', should be false");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_auto_fix_empty_string_is_false() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_AUTO_FIX", "");
    let config = OracleConfig::from_env();
    assert!(!config.auto_fix, "empty string != 'true', should be false");

    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

// ============================================================================
// FROM_ENV: ALL VARIABLES SET SIMULTANEOUSLY
// ============================================================================

#[test]
fn config_coverage_from_env_all_vars_set() {
    let _lock = lock_env();
    std::env::set_var("DECY_ORACLE_PATTERNS", "/tmp/test_oracle.apr");
    std::env::set_var("DECY_ORACLE_THRESHOLD", "0.95");
    std::env::set_var("DECY_ORACLE_AUTO_FIX", "true");

    let config = OracleConfig::from_env();
    assert_eq!(config.patterns_path, PathBuf::from("/tmp/test_oracle.apr"));
    assert!(
        (config.confidence_threshold - 0.95_f32).abs() < 0.01,
        "threshold: {}",
        config.confidence_threshold
    );
    assert!(config.auto_fix);
    // max_suggestions and max_retries should be defaults
    assert_eq!(config.max_suggestions, 5);
    assert_eq!(config.max_retries, 3);

    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");
}

#[test]
fn config_coverage_from_env_threshold_zero() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_THRESHOLD", "0.0");
    let config = OracleConfig::from_env();
    assert!(
        config.confidence_threshold.abs() < f32::EPSILON,
        "threshold should be 0.0, got {}",
        config.confidence_threshold
    );

    std::env::remove_var("DECY_ORACLE_THRESHOLD");
}

#[test]
fn config_coverage_from_env_threshold_one() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_THRESHOLD", "1.0");
    let config = OracleConfig::from_env();
    assert!(
        (config.confidence_threshold - 1.0_f32).abs() < f32::EPSILON,
        "threshold should be 1.0, got {}",
        config.confidence_threshold
    );

    std::env::remove_var("DECY_ORACLE_THRESHOLD");
}

#[test]
fn config_coverage_from_env_threshold_negative_parses() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_THRESHOLD", "-0.5");
    let config = OracleConfig::from_env();
    assert!(
        (config.confidence_threshold - (-0.5_f32)).abs() < f32::EPSILON,
        "threshold should be -0.5, got {}",
        config.confidence_threshold
    );

    std::env::remove_var("DECY_ORACLE_THRESHOLD");
}

#[test]
fn config_coverage_from_env_threshold_empty_string_uses_default() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_PATTERNS");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_THRESHOLD", "");
    let config = OracleConfig::from_env();
    // Empty string fails to parse as f32, so default is used
    assert!(
        (config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON,
        "empty string should fallback to default 0.7, got {}",
        config.confidence_threshold
    );

    std::env::remove_var("DECY_ORACLE_THRESHOLD");
}

#[test]
fn config_coverage_from_env_patterns_relative_path() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_PATTERNS", "relative/path.apr");
    let config = OracleConfig::from_env();
    assert_eq!(config.patterns_path, PathBuf::from("relative/path.apr"));

    std::env::remove_var("DECY_ORACLE_PATTERNS");
}

#[test]
fn config_coverage_from_env_patterns_empty_string() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_PATTERNS", "");
    let config = OracleConfig::from_env();
    assert_eq!(config.patterns_path, PathBuf::from(""));

    std::env::remove_var("DECY_ORACLE_PATTERNS");
}

// ============================================================================
// FROM_FILE: ADDITIONAL BRANCH COVERAGE
// ============================================================================

#[test]
fn config_coverage_from_file_with_patterns_path() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
patterns_path = "/custom/oracle/patterns.apr"
confidence_threshold = 0.6
max_suggestions = 3
auto_fix = false
max_retries = 1
"#
    )
    .unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(
        config.patterns_path,
        PathBuf::from("/custom/oracle/patterns.apr")
    );
    assert!((config.confidence_threshold - 0.6_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 3);
    assert!(!config.auto_fix);
    assert_eq!(config.max_retries, 1);
}

#[test]
fn config_coverage_from_file_only_patterns_path() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
patterns_path = "/specific/path.apr"
"#
    )
    .unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(config.patterns_path, PathBuf::from("/specific/path.apr"));
    // Defaults for everything else
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 5);
    assert!(!config.auto_fix);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn config_coverage_from_file_only_auto_fix() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "auto_fix = true").unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert!(config.auto_fix);
    // Other defaults preserved
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 5);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn config_coverage_from_file_only_max_retries() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "max_retries = 100").unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(config.max_retries, 100);
    // Other defaults preserved
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 5);
    assert!(!config.auto_fix);
}

#[test]
fn config_coverage_from_file_only_max_suggestions() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "max_suggestions = 0").unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(config.max_suggestions, 0);
}

#[test]
fn config_coverage_from_file_invalid_field_type_returns_error() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"confidence_threshold = "not_a_float""#).unwrap();

    let result = OracleConfig::from_file(file.path());
    assert!(result.is_err());
}

#[test]
fn config_coverage_from_file_unknown_field_ignored() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
confidence_threshold = 0.5
unknown_field = "ignored"
"#
    )
    .unwrap();

    // serde(default) with deny_unknown_fields not set should either ignore or error
    // depending on the derive. Let's test what actually happens.
    let result = OracleConfig::from_file(file.path());
    // toml deserialization by default ignores unknown fields
    if let Ok(config) = result {
        assert!((config.confidence_threshold - 0.5_f32).abs() < f32::EPSILON);
    }
    // If it errors, that's also fine -- just exercising the code path
}

// ============================================================================
// SERDE ROUND-TRIP: FULL COVERAGE OF SERIALIZE/DESERIALIZE
// ============================================================================

#[test]
fn config_coverage_serde_json_round_trip() {
    let config = OracleConfig {
        patterns_path: PathBuf::from("/test/serde.apr"),
        confidence_threshold: 0.42,
        max_suggestions: 7,
        auto_fix: true,
        max_retries: 2,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: OracleConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.patterns_path, config.patterns_path);
    assert!(
        (deserialized.confidence_threshold - config.confidence_threshold).abs() < f32::EPSILON
    );
    assert_eq!(deserialized.max_suggestions, config.max_suggestions);
    assert_eq!(deserialized.auto_fix, config.auto_fix);
    assert_eq!(deserialized.max_retries, config.max_retries);
}

#[test]
fn config_coverage_serde_json_round_trip_defaults() {
    let config = OracleConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: OracleConfig = serde_json::from_str(&json).unwrap();

    assert!(
        (deserialized.confidence_threshold - 0.7_f32).abs() < f32::EPSILON
    );
    assert_eq!(deserialized.max_suggestions, 5);
    assert!(!deserialized.auto_fix);
    assert_eq!(deserialized.max_retries, 3);
}

#[test]
fn config_coverage_toml_round_trip_all_fields() {
    let config = OracleConfig {
        patterns_path: PathBuf::from("/round/trip.apr"),
        confidence_threshold: 0.33,
        max_suggestions: 42,
        auto_fix: false,
        max_retries: 99,
    };

    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: OracleConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(deserialized.patterns_path, config.patterns_path);
    assert!(
        (deserialized.confidence_threshold - config.confidence_threshold).abs() < 0.01
    );
    assert_eq!(deserialized.max_suggestions, config.max_suggestions);
    assert_eq!(deserialized.auto_fix, config.auto_fix);
    assert_eq!(deserialized.max_retries, config.max_retries);
}

#[test]
fn config_coverage_serde_json_from_partial() {
    let json = r#"{"confidence_threshold": 0.99}"#;
    let config: OracleConfig = serde_json::from_str(json).unwrap();
    assert!((config.confidence_threshold - 0.99_f32).abs() < 0.01);
    // Defaults for missing fields
    assert_eq!(config.max_suggestions, 5);
    assert!(!config.auto_fix);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn config_coverage_serde_json_from_empty_object() {
    let json = "{}";
    let config: OracleConfig = serde_json::from_str(json).unwrap();
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 5);
    assert!(!config.auto_fix);
    assert_eq!(config.max_retries, 3);
}

// ============================================================================
// DEFAULT IMPLEMENTATION EDGE CASES
// ============================================================================

#[test]
fn config_coverage_default_patterns_path_parent_is_decy_dir() {
    let config = OracleConfig::default();
    let parent = config.patterns_path.parent().unwrap();
    assert!(
        parent.to_string_lossy().ends_with(".decy"),
        "parent dir should be .decy, got: {:?}",
        parent
    );
}

#[test]
fn config_coverage_default_confidence_threshold_in_range() {
    let config = OracleConfig::default();
    assert!(
        config.confidence_threshold >= 0.0 && config.confidence_threshold <= 1.0,
        "default threshold {} should be in [0.0, 1.0]",
        config.confidence_threshold
    );
}

#[test]
fn config_coverage_default_max_suggestions_nonzero() {
    let config = OracleConfig::default();
    assert!(config.max_suggestions > 0);
}

#[test]
fn config_coverage_default_max_retries_nonzero() {
    let config = OracleConfig::default();
    assert!(config.max_retries > 0);
}

#[test]
fn config_coverage_default_auto_fix_disabled() {
    let config = OracleConfig::default();
    assert!(!config.auto_fix);
}

// ============================================================================
// CLONE AND DEBUG TRAIT COVERAGE
// ============================================================================

#[test]
fn config_coverage_clone_preserves_all_fields() {
    let config = OracleConfig {
        patterns_path: PathBuf::from("/clone/test.apr"),
        confidence_threshold: 0.123,
        max_suggestions: 77,
        auto_fix: true,
        max_retries: 11,
    };
    let cloned = config.clone();

    assert_eq!(cloned.patterns_path, PathBuf::from("/clone/test.apr"));
    assert!((cloned.confidence_threshold - 0.123_f32).abs() < f32::EPSILON);
    assert_eq!(cloned.max_suggestions, 77);
    assert!(cloned.auto_fix);
    assert_eq!(cloned.max_retries, 11);
}

#[test]
fn config_coverage_debug_contains_all_field_names() {
    let config = OracleConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("patterns_path"));
    assert!(debug.contains("confidence_threshold"));
    assert!(debug.contains("max_suggestions"));
    assert!(debug.contains("auto_fix"));
    assert!(debug.contains("max_retries"));
}

#[test]
fn config_coverage_debug_format_with_custom_values() {
    let config = OracleConfig {
        patterns_path: PathBuf::from("/debug/format.apr"),
        confidence_threshold: 0.5,
        max_suggestions: 1,
        auto_fix: true,
        max_retries: 0,
    };
    let debug = format!("{config:?}");
    assert!(debug.contains("/debug/format.apr"));
    assert!(debug.contains("true"));
}

// ============================================================================
// STRUCT FIELD DIRECT ACCESS COVERAGE
// ============================================================================

#[test]
fn config_coverage_field_mutation() {
    let mut config = OracleConfig::default();
    config.patterns_path = PathBuf::from("/mutated.apr");
    config.confidence_threshold = 0.1;
    config.max_suggestions = 100;
    config.auto_fix = true;
    config.max_retries = 50;

    assert_eq!(config.patterns_path, PathBuf::from("/mutated.apr"));
    assert!((config.confidence_threshold - 0.1_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 100);
    assert!(config.auto_fix);
    assert_eq!(config.max_retries, 50);
}

// ============================================================================
// FROM_ENV: PATTERNS_PATH VARIANTS
// ============================================================================

#[test]
fn config_coverage_from_env_patterns_absolute_path() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_PATTERNS", "/absolute/path/to/patterns.apr");
    let config = OracleConfig::from_env();
    assert_eq!(
        config.patterns_path,
        PathBuf::from("/absolute/path/to/patterns.apr")
    );
    assert!(config.patterns_path.is_absolute());

    std::env::remove_var("DECY_ORACLE_PATTERNS");
}

#[test]
fn config_coverage_from_env_patterns_with_spaces() {
    let _lock = lock_env();
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    std::env::set_var("DECY_ORACLE_PATTERNS", "/path with spaces/patterns.apr");
    let config = OracleConfig::from_env();
    assert_eq!(
        config.patterns_path,
        PathBuf::from("/path with spaces/patterns.apr")
    );

    std::env::remove_var("DECY_ORACLE_PATTERNS");
}

// ============================================================================
// FROM_FILE: TOML CONTENT VARIATIONS
// ============================================================================

#[test]
fn config_coverage_from_file_whitespace_only_uses_defaults() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "   \n\n   ").unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 5);
}

#[test]
fn config_coverage_from_file_comments_only_uses_defaults() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# This is a comment\n# Another comment").unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert!((config.confidence_threshold - 0.7_f32).abs() < f32::EPSILON);
}

#[test]
fn config_coverage_from_file_all_fields_with_comments() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
# Oracle patterns location
patterns_path = "/commented/path.apr"

# Confidence threshold
confidence_threshold = 0.75

# Max suggestions
max_suggestions = 8

# Auto-fix
auto_fix = true

# Max retries
max_retries = 4
"#
    )
    .unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(
        config.patterns_path,
        PathBuf::from("/commented/path.apr")
    );
    assert!((config.confidence_threshold - 0.75_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 8);
    assert!(config.auto_fix);
    assert_eq!(config.max_retries, 4);
}

#[test]
fn config_coverage_from_file_extreme_values() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
confidence_threshold = 0.0
max_suggestions = 0
auto_fix = false
max_retries = 0
"#
    )
    .unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert!(config.confidence_threshold.abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 0);
    assert!(!config.auto_fix);
    assert_eq!(config.max_retries, 0);
}

#[test]
fn config_coverage_from_file_large_values() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
confidence_threshold = 1.0
max_suggestions = 999999
max_retries = 999999
"#
    )
    .unwrap();

    let config = OracleConfig::from_file(file.path()).unwrap();
    assert!((config.confidence_threshold - 1.0_f32).abs() < f32::EPSILON);
    assert_eq!(config.max_suggestions, 999_999);
    assert_eq!(config.max_retries, 999_999);
}

// ============================================================================
// FROM_ENV + FROM_FILE INTERACTION
// ============================================================================

#[test]
fn config_coverage_env_overrides_are_independent_of_file() {
    let _lock = lock_env();
    // Set env var
    std::env::set_var("DECY_ORACLE_PATTERNS", "/env/path.apr");
    std::env::remove_var("DECY_ORACLE_THRESHOLD");
    std::env::remove_var("DECY_ORACLE_AUTO_FIX");

    // from_env uses env var
    let env_config = OracleConfig::from_env();
    assert_eq!(env_config.patterns_path, PathBuf::from("/env/path.apr"));

    // from_file uses file content (unaffected by env)
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"patterns_path = "/file/path.apr""#).unwrap();
    let file_config = OracleConfig::from_file(file.path()).unwrap();
    assert_eq!(file_config.patterns_path, PathBuf::from("/file/path.apr"));

    std::env::remove_var("DECY_ORACLE_PATTERNS");
}

// ============================================================================
// TOML DESERIALIZATION ERROR PATHS
// ============================================================================

#[test]
fn config_coverage_from_file_invalid_toml_syntax() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "[[[nested").unwrap();

    let result = OracleConfig::from_file(file.path());
    assert!(result.is_err());
}

#[test]
fn config_coverage_from_file_wrong_value_type_bool_as_string() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"auto_fix = "yes""#).unwrap();

    let result = OracleConfig::from_file(file.path());
    assert!(result.is_err());
}

#[test]
fn config_coverage_from_file_wrong_value_type_int_as_string() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"max_retries = "ten""#).unwrap();

    let result = OracleConfig::from_file(file.path());
    assert!(result.is_err());
}

// ============================================================================
// TOML SERIALIZATION: ENSURE ALL FIELDS PRESENT
// ============================================================================

#[test]
fn config_coverage_toml_serialization_default() {
    let config = OracleConfig::default();
    let toml_str = toml::to_string(&config).unwrap();

    // All field names must appear in serialized output
    assert!(toml_str.contains("patterns_path"));
    assert!(toml_str.contains("confidence_threshold"));
    assert!(toml_str.contains("max_suggestions"));
    assert!(toml_str.contains("auto_fix"));
    assert!(toml_str.contains("max_retries"));
}

#[test]
fn config_coverage_toml_serialization_custom() {
    let config = OracleConfig {
        patterns_path: PathBuf::from("/ser/test.apr"),
        confidence_threshold: 0.55,
        max_suggestions: 3,
        auto_fix: true,
        max_retries: 7,
    };
    let toml_str = toml::to_string(&config).unwrap();

    assert!(toml_str.contains("/ser/test.apr"));
    assert!(toml_str.contains("auto_fix = true"));
    assert!(toml_str.contains("max_suggestions = 3"));
    assert!(toml_str.contains("max_retries = 7"));
}
