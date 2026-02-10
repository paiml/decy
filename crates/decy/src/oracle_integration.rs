//! Oracle integration for CITL-enhanced transpilation
//!
//! This module provides the bridge between decy CLI and decy-oracle,
//! implementing the Jidoka (automation with human intelligence) principle
//! from the Toyota Way.

#[cfg(feature = "citl")]
use decy_oracle::{
    CConstruct, CDecisionCategory, CDecisionContext, DecyOracle, OracleConfig, RustcError,
};

/// Oracle-related CLI options
#[derive(Debug, Clone, Default)]
pub struct OracleOptions {
    /// Enable oracle-assisted transpilation
    pub enabled: bool,
    /// Confidence threshold for auto-fix (0.0-1.0)
    #[cfg_attr(not(feature = "citl"), allow(dead_code))]
    pub threshold: f32,
    /// Enable automatic fix application
    #[cfg_attr(not(feature = "citl"), allow(dead_code))]
    pub auto_fix: bool,
    /// Maximum retry attempts
    #[cfg_attr(not(feature = "citl"), allow(dead_code))]
    pub max_retries: usize,
    /// Path to patterns file
    #[cfg_attr(not(feature = "citl"), allow(dead_code))]
    pub patterns_path: Option<std::path::PathBuf>,
    /// Enable pattern capture for learning
    pub capture_patterns: bool,
    /// Path to import patterns from (cross-project transfer)
    pub import_patterns_path: Option<std::path::PathBuf>,
    /// Report format (json, markdown, prometheus)
    pub report_format: Option<String>,
}

impl OracleOptions {
    /// Create options from CLI arguments
    pub fn new(enabled: bool, threshold: Option<f32>, auto_fix: bool) -> Self {
        Self {
            enabled,
            threshold: threshold.unwrap_or(0.7),
            auto_fix,
            max_retries: 3,
            patterns_path: None,
            capture_patterns: false,
            import_patterns_path: None,
            report_format: None,
        }
    }

    /// Create options with pattern capture enabled
    pub fn with_capture(mut self, capture: bool) -> Self {
        self.capture_patterns = capture;
        self
    }

    /// Create options with pattern import path
    pub fn with_import(mut self, path: Option<std::path::PathBuf>) -> Self {
        self.import_patterns_path = path;
        self
    }

    /// Create options with report format
    pub fn with_report_format(mut self, format: Option<String>) -> Self {
        self.report_format = format;
        self
    }

    /// Check if oracle is enabled and should be used
    pub fn should_use_oracle(&self) -> bool {
        self.enabled
    }

    /// Check if pattern capture is enabled
    #[cfg(test)]
    pub fn should_capture(&self) -> bool {
        self.capture_patterns
    }
}

/// Result of oracle-assisted transpilation
#[derive(Debug)]
pub struct OracleTranspileResult {
    /// Final Rust code
    pub rust_code: String,
    /// Number of oracle queries made
    pub oracle_queries: usize,
    /// Number of fixes applied
    pub fixes_applied: usize,
    /// Number of retries used
    pub retries_used: usize,
    /// Whether compilation succeeded
    pub compilation_success: bool,
    /// Remaining errors (if any)
    pub remaining_errors: Vec<String>,
    /// Number of patterns captured for learning
    pub patterns_captured: usize,
    /// Number of patterns imported from another project
    pub patterns_imported: usize,
}

/// Parse rustc error output into structured errors
#[cfg(feature = "citl")]
pub fn parse_rustc_errors(stderr: &str) -> Vec<RustcError> {
    let mut errors = Vec::new();

    // Parse error lines like: "error[E0382]: borrow of moved value"
    for line in stderr.lines() {
        if let Some(start) = line.find("error[E") {
            if let Some(end) = line[start..].find(']') {
                let code = &line[start + 6..start + end];
                let message = line[start + end + 2..].trim();
                errors.push(RustcError::new(code, message));
            }
        }
    }

    errors
}

/// Create decision context from C code construct
#[cfg(feature = "citl")]
pub fn create_context_for_error(
    _error: &RustcError,
    _c_code: &str,
    _rust_code: &str,
) -> CDecisionContext {
    // Default context - in production this would analyze the code
    CDecisionContext::new(
        CConstruct::RawPointer {
            is_const: false,
            pointee: "void".into(),
        },
        CDecisionCategory::PointerOwnership,
    )
}

/// Transpile with oracle assistance (requires citl feature for full functionality)
#[cfg(feature = "citl")]
pub fn transpile_with_oracle(
    c_code: &str,
    options: &OracleOptions,
) -> anyhow::Result<OracleTranspileResult> {
    use anyhow::Context;

    // Initialize oracle
    let config = if let Some(ref path) = options.patterns_path {
        OracleConfig {
            patterns_path: path.clone(),
            confidence_threshold: options.threshold,
            auto_fix: options.auto_fix,
            max_retries: options.max_retries,
            ..Default::default()
        }
    } else {
        OracleConfig {
            confidence_threshold: options.threshold,
            auto_fix: options.auto_fix,
            max_retries: options.max_retries,
            ..Default::default()
        }
    };

    let mut oracle = DecyOracle::new(config).context("Failed to initialize oracle")?;

    // Import patterns from another project if specified
    let patterns_imported = if let Some(ref import_path) = options.import_patterns_path {
        oracle.import_patterns(import_path).unwrap_or(0)
    } else {
        0
    };

    // Initial transpilation
    let mut rust_code = decy_core::transpile(c_code).context("Initial transpilation failed")?;

    let mut result = OracleTranspileResult {
        rust_code: rust_code.clone(),
        oracle_queries: 0,
        fixes_applied: 0,
        retries_used: 0,
        compilation_success: false,
        remaining_errors: Vec::new(),
        patterns_captured: 0,
        patterns_imported,
    };

    // Track errors with pending fix verification
    let mut pending_verified: Vec<RustcError> = Vec::new();

    // Oracle feedback loop
    for retry in 0..options.max_retries {
        // Check compilation
        match check_rust_compilation(&rust_code) {
            Ok(()) => {
                // Compilation succeeded - verify pending fixes
                for error in &pending_verified {
                    oracle.record_fix_verified(error);
                    result.patterns_captured += 1;
                }

                // Save patterns if capture is enabled
                if options.capture_patterns && !pending_verified.is_empty() {
                    let _ = oracle.save(); // Best-effort save
                }

                result.compilation_success = true;
                result.rust_code = rust_code;
                return Ok(result);
            }
            Err(stderr) => {
                let errors = parse_rustc_errors(&stderr);

                if errors.is_empty() {
                    // Non-standard error format
                    result.remaining_errors = vec![stderr];
                    result.rust_code = rust_code;
                    return Ok(result);
                }

                // Clear pending - previous fixes didn't fully resolve issues
                pending_verified.clear();

                // Try oracle for each error
                let mut any_fix_applied = false;
                for error in &errors {
                    result.oracle_queries += 1;

                    let context = create_context_for_error(error, c_code, &rust_code);

                    if let Some(suggestion) = oracle.suggest_fix(error, &context) {
                        if options.auto_fix {
                            // Apply the fix
                            if let Ok(fixed) = apply_fix(&rust_code, &suggestion.pattern.fix_diff) {
                                rust_code = fixed;
                                result.fixes_applied += 1;
                                any_fix_applied = true;
                                oracle.record_fix_applied(error);
                                // Track for verification on next compile
                                pending_verified.push(error.clone());
                            }
                        }
                    } else {
                        oracle.record_miss(error);
                    }
                }

                result.retries_used = retry + 1;

                if !any_fix_applied {
                    // No more fixes available
                    result.remaining_errors = errors.iter().map(|e| e.message.clone()).collect();
                    break;
                }
            }
        }
    }

    result.rust_code = rust_code;
    Ok(result)
}

/// Stub for non-citl builds (basic transpilation without oracle assistance)
#[cfg(not(feature = "citl"))]
pub fn transpile_with_oracle(
    c_code: &str,
    _options: &OracleOptions,
) -> anyhow::Result<OracleTranspileResult> {
    let rust_code = decy_core::transpile(c_code)?;
    Ok(OracleTranspileResult {
        rust_code,
        oracle_queries: 0,
        fixes_applied: 0,
        retries_used: 0,
        compilation_success: false,
        remaining_errors: vec!["CITL feature not enabled - basic transpilation only".into()],
        patterns_captured: 0,
        patterns_imported: 0,
    })
}

/// Check if Rust code compiles
#[cfg(feature = "citl")]
fn check_rust_compilation(rust_code: &str) -> Result<(), String> {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use unique temp files to avoid race conditions
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("decy_oracle_check_{}.rs", unique_id));
    let temp_output = temp_dir.join(format!("decy_oracle_check_{}.rmeta", unique_id));

    std::fs::write(&temp_file, rust_code)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Run rustc --emit=metadata (fast check, no codegen)
    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--edition=2021")
        .arg("-o")
        .arg(&temp_output)
        .arg(&temp_file)
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    // Clean up
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(&temp_output);

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Apply a unified diff to Rust code
#[cfg(feature = "citl")]
fn apply_fix(rust_code: &str, diff: &str) -> Result<String, String> {
    // Simple line-based diff application
    // Format: "- old line\n+ new line"
    let mut result = rust_code.to_string();

    let lines: Vec<&str> = diff.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if let Some(old) = line.strip_prefix("- ") {
            if i + 1 < lines.len() {
                if let Some(new) = lines[i + 1].strip_prefix("+ ") {
                    result = result.replace(old, new);
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ORACLE OPTIONS CONSTRUCTION TESTS
    // ============================================================================

    #[test]
    fn test_oracle_options_default() {
        let opts = OracleOptions::default();
        assert!(!opts.enabled);
        assert!(!opts.auto_fix);
        assert_eq!(opts.max_retries, 0);
        assert!(opts.patterns_path.is_none());
        assert!(!opts.capture_patterns);
        assert!(opts.import_patterns_path.is_none());
        assert!(opts.report_format.is_none());
    }

    #[test]
    fn test_oracle_options_new() {
        let opts = OracleOptions::new(true, Some(0.8), true);
        assert!(opts.enabled);
        assert!(opts.auto_fix);
        assert!((opts.threshold - 0.8).abs() < f32::EPSILON);
        assert_eq!(opts.max_retries, 3);
    }

    #[test]
    fn test_oracle_options_new_default_threshold() {
        let opts = OracleOptions::new(true, None, false);
        assert!((opts.threshold - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oracle_options_debug() {
        let opts = OracleOptions::new(true, Some(0.9), true);
        let debug_str = format!("{:?}", opts);
        assert!(debug_str.contains("OracleOptions"));
        assert!(debug_str.contains("enabled: true"));
    }

    #[test]
    fn test_oracle_options_clone() {
        let opts = OracleOptions::new(true, Some(0.85), true)
            .with_capture(true)
            .with_report_format(Some("json".into()));
        let cloned = opts.clone();
        assert_eq!(cloned.enabled, opts.enabled);
        assert!((cloned.threshold - opts.threshold).abs() < f32::EPSILON);
        assert_eq!(cloned.auto_fix, opts.auto_fix);
        assert_eq!(cloned.capture_patterns, opts.capture_patterns);
        assert_eq!(cloned.report_format, opts.report_format);
    }

    // ============================================================================
    // ORACLE OPTIONS BUILDER TESTS
    // ============================================================================

    #[test]
    fn test_should_use_oracle() {
        let opts = OracleOptions::new(true, None, false);
        assert!(opts.should_use_oracle());

        let opts = OracleOptions::new(false, None, false);
        assert!(!opts.should_use_oracle());
    }

    #[test]
    fn test_with_capture() {
        let opts = OracleOptions::new(true, None, false).with_capture(true);
        assert!(opts.should_capture());
        assert!(opts.capture_patterns);

        let opts = OracleOptions::new(true, None, false).with_capture(false);
        assert!(!opts.should_capture());
    }

    #[test]
    fn test_capture_patterns_default() {
        let opts = OracleOptions::default();
        assert!(!opts.capture_patterns);
        assert!(!opts.should_capture());
    }

    #[test]
    fn test_with_import() {
        let path = std::path::PathBuf::from("/tmp/patterns.apr");
        let opts = OracleOptions::new(true, None, false).with_import(Some(path.clone()));
        assert_eq!(opts.import_patterns_path, Some(path));

        let opts = OracleOptions::new(true, None, false).with_import(None);
        assert!(opts.import_patterns_path.is_none());
    }

    #[test]
    fn test_with_report_format() {
        let opts = OracleOptions::new(true, None, false).with_report_format(Some("json".into()));
        assert_eq!(opts.report_format, Some("json".into()));

        let opts = OracleOptions::new(true, None, false).with_report_format(None);
        assert!(opts.report_format.is_none());
    }

    #[test]
    fn test_with_report_format_markdown() {
        let opts =
            OracleOptions::new(true, None, false).with_report_format(Some("markdown".into()));
        assert_eq!(opts.report_format, Some("markdown".into()));
    }

    #[test]
    fn test_with_report_format_prometheus() {
        let opts =
            OracleOptions::new(true, None, false).with_report_format(Some("prometheus".into()));
        assert_eq!(opts.report_format, Some("prometheus".into()));
    }

    #[test]
    fn test_builder_chaining() {
        let path = std::path::PathBuf::from("/test/import.apr");
        let opts = OracleOptions::new(true, Some(0.9), true)
            .with_capture(true)
            .with_import(Some(path.clone()))
            .with_report_format(Some("json".into()));

        assert!(opts.enabled);
        assert!(opts.auto_fix);
        assert!((opts.threshold - 0.9).abs() < f32::EPSILON);
        assert!(opts.capture_patterns);
        assert_eq!(opts.import_patterns_path, Some(path));
        assert_eq!(opts.report_format, Some("json".into()));
    }

    // ============================================================================
    // ORACLE TRANSPILE RESULT TESTS
    // ============================================================================

    #[test]
    fn test_transpile_result_struct() {
        let result = OracleTranspileResult {
            rust_code: "fn main() {}".into(),
            oracle_queries: 5,
            fixes_applied: 2,
            retries_used: 1,
            compilation_success: true,
            remaining_errors: vec![],
            patterns_captured: 3,
            patterns_imported: 7,
        };

        assert!(result.compilation_success);
        assert_eq!(result.oracle_queries, 5);
        assert_eq!(result.fixes_applied, 2);
        assert_eq!(result.patterns_captured, 3);
        assert_eq!(result.patterns_imported, 7);
    }

    #[test]
    fn test_transpile_result_with_errors() {
        let result = OracleTranspileResult {
            rust_code: "fn main() { let x = moved; }".into(),
            oracle_queries: 10,
            fixes_applied: 0,
            retries_used: 3,
            compilation_success: false,
            remaining_errors: vec!["E0382: borrow of moved value".into()],
            patterns_captured: 0,
            patterns_imported: 0,
        };

        assert!(!result.compilation_success);
        assert_eq!(result.remaining_errors.len(), 1);
        assert!(result.remaining_errors[0].contains("E0382"));
    }

    #[test]
    fn test_transpile_result_debug() {
        let result = OracleTranspileResult {
            rust_code: "test".into(),
            oracle_queries: 1,
            fixes_applied: 0,
            retries_used: 0,
            compilation_success: false,
            remaining_errors: vec![],
            patterns_captured: 0,
            patterns_imported: 0,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("OracleTranspileResult"));
    }

    // ============================================================================
    // TRANSPILE WITH ORACLE (NON-CITL) TESTS
    // ============================================================================

    #[test]
    #[cfg(not(feature = "citl"))]
    fn test_transpile_with_oracle_non_citl() {
        let c_code = "int main() { return 0; }";
        let opts = OracleOptions::new(true, None, false);
        let result = transpile_with_oracle(c_code, &opts);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.compilation_success);
        assert_eq!(result.oracle_queries, 0);
        assert_eq!(result.fixes_applied, 0);
        assert!(result.remaining_errors[0].contains("CITL feature not enabled"));
    }

    #[test]
    #[cfg(not(feature = "citl"))]
    fn test_transpile_with_oracle_non_citl_outputs_rust_code() {
        let c_code = "void hello() {}";
        let opts = OracleOptions::default();
        let result = transpile_with_oracle(c_code, &opts);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.rust_code.contains("fn hello"));
    }

    #[test]
    #[cfg(not(feature = "citl"))]
    fn test_transpile_with_oracle_non_citl_invalid_code() {
        let c_code = "invalid syntax {{{";
        let opts = OracleOptions::default();
        let result = transpile_with_oracle(c_code, &opts);

        assert!(result.is_err());
    }

    // ============================================================================
    // CITL FEATURE TESTS
    // ============================================================================

    #[test]
    #[cfg(feature = "citl")]
    fn test_parse_rustc_errors() {
        let stderr = r#"error[E0382]: borrow of moved value: `x`
   --> test.rs:5:10
    |
5   |     let y = x;
    |             - value moved here
6   |     println!("{}", x);
    |                    ^ value borrowed here after move

error[E0499]: cannot borrow `data` as mutable more than once
   --> test.rs:10:5
"#;

        let errors = parse_rustc_errors(stderr);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].code, "E0382");
        assert_eq!(errors[1].code, "E0499");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_parse_rustc_errors_empty() {
        let stderr = "";
        let errors = parse_rustc_errors(stderr);
        assert!(errors.is_empty());
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_parse_rustc_errors_no_errors() {
        let stderr = "warning: unused variable `x`";
        let errors = parse_rustc_errors(stderr);
        assert!(errors.is_empty());
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_parse_rustc_errors_single() {
        let stderr = "error[E0001]: test error message";
        let errors = parse_rustc_errors(stderr);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E0001");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_apply_fix() {
        let rust_code = "let x = value.clone();";
        let diff = "- value.clone()\n+ value.to_owned()";

        let result = apply_fix(rust_code, diff).unwrap();
        assert_eq!(result, "let x = value.to_owned();");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_apply_fix_no_match() {
        let rust_code = "let x = something_else();";
        let diff = "- value.clone()\n+ value.to_owned()";

        let result = apply_fix(rust_code, diff).unwrap();
        // No change when diff doesn't match
        assert_eq!(result, "let x = something_else();");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_apply_fix_empty_diff() {
        let rust_code = "let x = value;";
        let diff = "";

        let result = apply_fix(rust_code, diff).unwrap();
        assert_eq!(result, "let x = value;");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_apply_fix_multiple_replacements() {
        let rust_code = "let a = x.clone(); let b = x.clone();";
        let diff = "- x.clone()\n+ x.to_owned()";

        let result = apply_fix(rust_code, diff).unwrap();
        // Both occurrences should be replaced
        assert_eq!(result, "let a = x.to_owned(); let b = x.to_owned();");
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_create_context_for_error() {
        let error = RustcError::new("E0382", "borrow of moved value");
        let c_code = "int* ptr;";
        let rust_code = "let ptr: *mut i32;";

        let context = create_context_for_error(&error, c_code, rust_code);
        // Should create a default context
        assert!(format!("{:?}", context).contains("CDecisionContext"));
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_check_rust_compilation_valid() {
        let valid_code = "fn main() {}";
        let result = check_rust_compilation(valid_code);
        if let Err(ref e) = result {
            eprintln!("Compilation error: {}", e);
        }
        // Skip test if rustc is not available
        if result
            .as_ref()
            .err()
            .map(|e| e.contains("Failed to run rustc"))
            .unwrap_or(false)
        {
            eprintln!("Skipping test: rustc not available");
            return;
        }
        assert!(
            result.is_ok(),
            "Expected compilation to succeed, got: {:?}",
            result
        );
    }

    #[test]
    #[cfg(feature = "citl")]
    fn test_check_rust_compilation_invalid() {
        let invalid_code = "fn main() { let x: i32 = \"not an int\"; }";
        let result = check_rust_compilation(invalid_code);
        assert!(result.is_err());
    }
}
