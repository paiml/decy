//! Coverage tests for verification.rs - targeting all uncovered branches.

use crate::verification::{
    check_rust_compilation, run_test_suite, verify_fix_semantically, TestResult,
    VerificationConfig, VerificationResult, VerificationStats,
};
use std::path::Path;

// ============================================================================
// VerificationResult - exhaustive branch coverage
// ============================================================================

#[test]
fn test_behavior_changed_allows_promotion_false() {
    let result = VerificationResult::BehaviorChanged(vec!["failure1".to_string()]);
    assert!(!result.allows_promotion());
}

#[test]
fn test_compile_failed_allows_promotion_false() {
    let result = VerificationResult::CompileFailed("type mismatch".to_string());
    assert!(!result.allows_promotion());
}

#[test]
fn test_fully_verified_is_not_compile_failure() {
    assert!(!VerificationResult::FullyVerified.is_compile_failure());
}

#[test]
fn test_compiles_only_is_not_compile_failure() {
    assert!(!VerificationResult::CompilesOnly.is_compile_failure());
}

#[test]
fn test_behavior_changed_is_not_compile_failure() {
    let result = VerificationResult::BehaviorChanged(vec!["test".to_string()]);
    assert!(!result.is_compile_failure());
}

#[test]
fn test_fully_verified_has_no_behavior_change() {
    assert!(!VerificationResult::FullyVerified.has_behavior_change());
}

#[test]
fn test_compiles_only_has_no_behavior_change() {
    assert!(!VerificationResult::CompilesOnly.has_behavior_change());
}

#[test]
fn test_behavior_changed_with_multiple_failures() {
    let failures = vec![
        "test_add failed".to_string(),
        "test_sub failed".to_string(),
        "test_mul failed".to_string(),
    ];
    let result = VerificationResult::BehaviorChanged(failures);
    assert!(result.has_behavior_change());
    assert!(!result.allows_promotion());
    assert_eq!(result.confidence_weight(), 0.0);
}

#[test]
fn test_compile_failed_with_detailed_error() {
    let result = VerificationResult::CompileFailed(
        "error[E0308]: mismatched types\n  --> src/main.rs:2:18".to_string(),
    );
    assert!(result.is_compile_failure());
    assert!(!result.allows_promotion());
    assert_eq!(result.confidence_weight(), 0.0);
}

// ============================================================================
// TestResult - all variant coverage
// ============================================================================

#[test]
fn test_some_failed_is_not_ok() {
    let result = TestResult::SomeFailed(vec!["test_foo".to_string()]);
    assert!(!result.is_ok());
}

#[test]
fn test_execution_error_is_not_ok() {
    let result = TestResult::ExecutionError("timeout".to_string());
    assert!(!result.is_ok());
}

#[test]
fn test_all_passed_is_ok() {
    assert!(TestResult::AllPassed.is_ok());
}

#[test]
fn test_no_tests_is_ok() {
    assert!(TestResult::NoTests.is_ok());
}

// ============================================================================
// VerificationConfig - branch coverage
// ============================================================================

#[test]
fn test_verification_config_custom_values() {
    let config = VerificationConfig {
        compile_timeout_secs: 30,
        test_timeout_secs: 60,
        run_tests: false,
        work_dir: Some(std::path::PathBuf::from("/tmp/decy_test")),
    };
    assert_eq!(config.compile_timeout_secs, 30);
    assert_eq!(config.test_timeout_secs, 60);
    assert!(!config.run_tests);
    assert!(config.work_dir.is_some());
}

#[test]
fn test_verification_config_with_work_dir() {
    let tmp = std::env::temp_dir();
    let config = VerificationConfig {
        work_dir: Some(tmp.clone()),
        ..Default::default()
    };
    assert_eq!(config.work_dir.unwrap(), tmp);
}

// ============================================================================
// VerificationStats - all method branches
// ============================================================================

#[test]
fn test_stats_record_all_variants() {
    let mut stats = VerificationStats::new();

    stats.record(&VerificationResult::FullyVerified);
    assert_eq!(stats.fully_verified, 1);
    assert_eq!(stats.total_evaluated, 1);

    stats.record(&VerificationResult::CompilesOnly);
    assert_eq!(stats.compiles_only, 1);
    assert_eq!(stats.total_evaluated, 2);

    stats.record(&VerificationResult::BehaviorChanged(vec!["fail".to_string()]));
    assert_eq!(stats.behavior_changed, 1);
    assert_eq!(stats.total_evaluated, 3);

    stats.record(&VerificationResult::CompileFailed("error".to_string()));
    assert_eq!(stats.compile_failed, 1);
    assert_eq!(stats.total_evaluated, 4);
}

#[test]
fn test_stats_promotion_rate_all_promoted() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::CompilesOnly);
    assert!((stats.promotion_rate() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_promotion_rate_none_promoted() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::CompileFailed("err".to_string()));
    stats.record(&VerificationResult::BehaviorChanged(vec![]));
    assert!((stats.promotion_rate() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_full_verification_rate_all_verified() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::FullyVerified);
    assert!((stats.full_verification_rate() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_full_verification_rate_none_verified() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::CompilesOnly);
    assert!((stats.full_verification_rate() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_average_confidence_mixed() {
    let mut stats = VerificationStats::new();
    // 1.0 + 0.6 + 0.0 + 0.0 = 1.6 / 4 = 0.4
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::CompilesOnly);
    stats.record(&VerificationResult::BehaviorChanged(vec![]));
    stats.record(&VerificationResult::CompileFailed("".to_string()));
    assert!((stats.average_confidence() - 0.4).abs() < 0.01);
}

#[test]
fn test_stats_average_confidence_all_fully_verified() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::FullyVerified);
    assert!((stats.average_confidence() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_serialization_roundtrip() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::CompilesOnly);
    stats.record(&VerificationResult::CompileFailed("err".to_string()));

    let json = serde_json::to_string(&stats).expect("serialize");
    let deserialized: VerificationStats = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.fully_verified, 1);
    assert_eq!(deserialized.compiles_only, 1);
    assert_eq!(deserialized.compile_failed, 1);
    assert_eq!(deserialized.total_evaluated, 3);
}

// ============================================================================
// check_rust_compilation - branch coverage
// ============================================================================

#[test]
fn test_compilation_valid_simple_function() {
    let code = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_ok());
}

#[test]
fn test_compilation_valid_empty_function() {
    let code = "pub fn noop() {}";
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_ok());
}

#[test]
fn test_compilation_invalid_undeclared_variable() {
    let code = "pub fn bad() -> i32 { undefined_var }";
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn test_compilation_invalid_missing_semicolon() {
    let code = "pub fn bad() { let x = 5 let y = 10; }";
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_err());
}

#[test]
fn test_compilation_with_custom_work_dir() {
    let tmp = std::env::temp_dir();
    let config = VerificationConfig {
        work_dir: Some(tmp),
        ..Default::default()
    };
    let code = "pub fn hello() -> &'static str { \"world\" }";
    let result = check_rust_compilation(code, &config);
    assert!(result.is_ok());
}

#[test]
fn test_compilation_invalid_code_with_custom_work_dir() {
    let tmp = std::env::temp_dir();
    let config = VerificationConfig {
        work_dir: Some(tmp),
        ..Default::default()
    };
    let code = "fn broken() -> i32 { \"not an int\" }";
    let result = check_rust_compilation(code, &config);
    assert!(result.is_err());
}

#[test]
fn test_compilation_with_warnings_still_passes() {
    // Unused variable generates a warning but should still compile
    let code = "pub fn warn_test() { let _unused_result = 42; }";
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_ok());
}

#[test]
fn test_compilation_complex_valid_code() {
    let code = r#"
        pub struct Point {
            pub x: f64,
            pub y: f64,
        }

        impl Point {
            pub fn distance(&self, other: &Point) -> f64 {
                ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
            }
        }
    "#;
    let config = VerificationConfig::default();
    let result = check_rust_compilation(code, &config);
    assert!(result.is_ok());
}

// ============================================================================
// run_test_suite - branch coverage
// ============================================================================

#[test]
fn test_run_test_suite_nonexistent_path() {
    let result = run_test_suite(Path::new("/nonexistent/path/to/tests"), "fn main() {}");
    assert_eq!(result, TestResult::NoTests);
}

#[test]
fn test_run_test_suite_file_not_dir() {
    // A file path (not a directory) should return NoTests via the is_dir() check
    let tmp = std::env::temp_dir().join("decy_verification_test_file.txt");
    std::fs::write(&tmp, "not a directory").ok();
    let result = run_test_suite(&tmp, "fn main() {}");
    assert_eq!(result, TestResult::NoTests);
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_run_test_suite_empty_dir() {
    // An empty directory should return NoTests
    let tmp = std::env::temp_dir().join("decy_verification_empty_test_dir");
    let _ = std::fs::create_dir_all(&tmp);
    // Make sure it's empty
    if let Ok(entries) = std::fs::read_dir(&tmp) {
        for entry in entries {
            if let Ok(e) = entry {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let result = run_test_suite(&tmp, "fn main() {}");
    assert_eq!(result, TestResult::NoTests);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_run_test_suite_dir_with_files_but_no_script() {
    // A directory with files but no run_tests.sh should return NoTests
    let tmp = std::env::temp_dir().join("decy_verification_no_script_dir");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("some_test.rs"), "fn test() {}").ok();
    let result = run_test_suite(&tmp, "fn main() {}");
    assert_eq!(result, TestResult::NoTests);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_run_test_suite_dir_with_passing_script() {
    // A directory with run_tests.sh that exits 0
    let tmp = std::env::temp_dir().join("decy_verification_pass_script_dir");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("placeholder.txt"), "test data").ok();
    std::fs::write(tmp.join("run_tests.sh"), "#!/bin/bash\nexit 0\n").ok();
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            tmp.join("run_tests.sh"),
            std::fs::Permissions::from_mode(0o755),
        );
    }
    let result = run_test_suite(&tmp, "fn main() {}");
    assert_eq!(result, TestResult::AllPassed);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_run_test_suite_dir_with_failing_script() {
    // A directory with run_tests.sh that exits non-zero
    let tmp = std::env::temp_dir().join("decy_verification_fail_script_dir");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("placeholder.txt"), "test data").ok();
    std::fs::write(
        tmp.join("run_tests.sh"),
        "#!/bin/bash\necho 'test failed' >&2\nexit 1\n",
    )
    .ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            tmp.join("run_tests.sh"),
            std::fs::Permissions::from_mode(0o755),
        );
    }
    let result = run_test_suite(&tmp, "fn main() {}");
    match result {
        TestResult::SomeFailed(failures) => {
            assert!(!failures.is_empty());
        }
        _ => panic!("Expected SomeFailed, got {:?}", result),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================================
// verify_fix_semantically - branch coverage
// ============================================================================

#[test]
fn test_verify_compile_failed_returns_compile_failed() {
    let code = "fn invalid() -> i32 { \"not_int\" }";
    let config = VerificationConfig::default();
    let result = verify_fix_semantically(code, None, &config);
    match result {
        VerificationResult::CompileFailed(msg) => {
            assert!(!msg.is_empty());
        }
        _ => panic!("Expected CompileFailed, got {:?}", result),
    }
}

#[test]
fn test_verify_compiles_only_when_tests_disabled() {
    let code = "pub fn ok() -> i32 { 42 }";
    let config = VerificationConfig {
        run_tests: false,
        ..Default::default()
    };
    let result = verify_fix_semantically(code, None, &config);
    assert_eq!(result, VerificationResult::CompilesOnly);
}

#[test]
fn test_verify_compiles_only_with_no_test_suite() {
    let code = "pub fn ok() -> i32 { 42 }";
    let config = VerificationConfig::default();
    let result = verify_fix_semantically(code, None, &config);
    assert_eq!(result, VerificationResult::CompilesOnly);
}

#[test]
fn test_verify_with_nonexistent_test_path() {
    let code = "pub fn ok() {}";
    let config = VerificationConfig::default();
    let test_path = Path::new("/tmp/nonexistent_verification_tests_12345");
    let result = verify_fix_semantically(code, Some(test_path), &config);
    // NoTests maps to CompilesOnly
    assert_eq!(result, VerificationResult::CompilesOnly);
}

#[test]
fn test_verify_fully_verified_with_passing_tests() {
    let tmp = std::env::temp_dir().join("decy_verify_full_pass");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("placeholder.txt"), "test data").ok();
    std::fs::write(tmp.join("run_tests.sh"), "#!/bin/bash\nexit 0\n").ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            tmp.join("run_tests.sh"),
            std::fs::Permissions::from_mode(0o755),
        );
    }
    let code = "pub fn good() -> i32 { 42 }";
    let config = VerificationConfig::default();
    let result = verify_fix_semantically(code, Some(tmp.as_path()), &config);
    assert_eq!(result, VerificationResult::FullyVerified);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_verify_behavior_changed_with_failing_tests() {
    let tmp = std::env::temp_dir().join("decy_verify_behavior_changed");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("placeholder.txt"), "test data").ok();
    std::fs::write(
        tmp.join("run_tests.sh"),
        "#!/bin/bash\necho 'assertion failed' >&2\nexit 1\n",
    )
    .ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            tmp.join("run_tests.sh"),
            std::fs::Permissions::from_mode(0o755),
        );
    }
    let code = "pub fn good() -> i32 { 42 }";
    let config = VerificationConfig::default();
    let result = verify_fix_semantically(code, Some(tmp.as_path()), &config);
    match result {
        VerificationResult::BehaviorChanged(failures) => {
            assert!(!failures.is_empty());
        }
        _ => panic!("Expected BehaviorChanged, got {:?}", result),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_verify_empty_test_dir_returns_compiles_only() {
    let tmp = std::env::temp_dir().join("decy_verify_empty_dir");
    let _ = std::fs::create_dir_all(&tmp);
    // Clean out any existing files
    if let Ok(entries) = std::fs::read_dir(&tmp) {
        for entry in entries {
            if let Ok(e) = entry {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let code = "pub fn good() -> i32 { 42 }";
    let config = VerificationConfig::default();
    let result = verify_fix_semantically(code, Some(tmp.as_path()), &config);
    // Empty dir => NoTests => CompilesOnly
    assert_eq!(result, VerificationResult::CompilesOnly);
    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================================
// VerificationResult equality (Clone + PartialEq derive coverage)
// ============================================================================

#[test]
fn test_verification_result_clone_fully_verified() {
    let result = VerificationResult::FullyVerified;
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn test_verification_result_clone_compiles_only() {
    let result = VerificationResult::CompilesOnly;
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn test_verification_result_clone_behavior_changed() {
    let result = VerificationResult::BehaviorChanged(vec!["fail".to_string()]);
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn test_verification_result_clone_compile_failed() {
    let result = VerificationResult::CompileFailed("error".to_string());
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn test_verification_result_debug_format() {
    let result = VerificationResult::FullyVerified;
    let debug = format!("{:?}", result);
    assert!(debug.contains("FullyVerified"));
}

#[test]
fn test_test_result_debug_format() {
    let result = TestResult::AllPassed;
    let debug = format!("{:?}", result);
    assert!(debug.contains("AllPassed"));
}

#[test]
fn test_verification_config_debug_format() {
    let config = VerificationConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("compile_timeout_secs"));
}

#[test]
fn test_verification_config_clone() {
    let config = VerificationConfig {
        compile_timeout_secs: 30,
        test_timeout_secs: 90,
        run_tests: false,
        work_dir: Some(std::path::PathBuf::from("/tmp")),
    };
    let cloned = config.clone();
    assert_eq!(cloned.compile_timeout_secs, 30);
    assert_eq!(cloned.test_timeout_secs, 90);
    assert!(!cloned.run_tests);
    assert_eq!(cloned.work_dir, Some(std::path::PathBuf::from("/tmp")));
}

// ============================================================================
// VerificationStats - Default + Serialize/Deserialize edge cases
// ============================================================================

#[test]
fn test_stats_default_is_zeroed() {
    let stats = VerificationStats::default();
    assert_eq!(stats.fully_verified, 0);
    assert_eq!(stats.compiles_only, 0);
    assert_eq!(stats.behavior_changed, 0);
    assert_eq!(stats.compile_failed, 0);
    assert_eq!(stats.total_evaluated, 0);
}

#[test]
fn test_stats_clone() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    let cloned = stats.clone();
    assert_eq!(cloned.fully_verified, 1);
    assert_eq!(cloned.total_evaluated, 1);
}

#[test]
fn test_stats_promotion_rate_single_verified() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    assert!((stats.promotion_rate() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_stats_full_verification_rate_mixed() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::FullyVerified);
    stats.record(&VerificationResult::CompilesOnly);
    stats.record(&VerificationResult::CompileFailed("".to_string()));
    // 1 fully verified out of 3
    assert!((stats.full_verification_rate() - 1.0 / 3.0).abs() < 0.01);
}

#[test]
fn test_stats_average_confidence_only_compiles_only() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::CompilesOnly);
    stats.record(&VerificationResult::CompilesOnly);
    // Average: (0.6 + 0.6) / 2 = 0.6
    assert!((stats.average_confidence() - 0.6).abs() < f32::EPSILON);
}

#[test]
fn test_stats_average_confidence_only_failures() {
    let mut stats = VerificationStats::new();
    stats.record(&VerificationResult::CompileFailed("e1".to_string()));
    stats.record(&VerificationResult::BehaviorChanged(vec!["e2".to_string()]));
    assert!((stats.average_confidence() - 0.0).abs() < f32::EPSILON);
}
