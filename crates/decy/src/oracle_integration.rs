//! Oracle integration for CITL-enhanced transpilation
//!
//! This module provides the bridge between decy CLI and decy-oracle,
//! implementing the Jidoka (automation with human intelligence) principle
//! from the Toyota Way.

#[cfg(feature = "oracle")]
use decy_oracle::{CDecisionCategory, CDecisionContext, CConstruct, DecyOracle, OracleConfig, RustcError};

/// Oracle-related CLI options
#[derive(Debug, Clone, Default)]
pub struct OracleOptions {
    /// Enable oracle-assisted transpilation
    pub enabled: bool,
    /// Confidence threshold for auto-fix (0.0-1.0)
    pub threshold: f32,
    /// Enable automatic fix application
    pub auto_fix: bool,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Path to patterns file
    pub patterns_path: Option<std::path::PathBuf>,
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
        }
    }

    /// Check if oracle is enabled and should be used
    pub fn should_use_oracle(&self) -> bool {
        self.enabled
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
}

/// Parse rustc error output into structured errors
#[cfg(feature = "oracle")]
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
#[cfg(feature = "oracle")]
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

/// Transpile with oracle assistance
#[cfg(feature = "oracle")]
pub fn transpile_with_oracle(
    c_code: &str,
    options: &OracleOptions,
) -> anyhow::Result<OracleTranspileResult> {
    use anyhow::Context;

    // Initialize oracle
    let config = if let Some(ref path) = options.patterns_path {
        let mut cfg = OracleConfig::default();
        cfg.patterns_path = path.clone();
        cfg.confidence_threshold = options.threshold;
        cfg.auto_fix = options.auto_fix;
        cfg.max_retries = options.max_retries;
        cfg
    } else {
        let mut cfg = OracleConfig::default();
        cfg.confidence_threshold = options.threshold;
        cfg.auto_fix = options.auto_fix;
        cfg.max_retries = options.max_retries;
        cfg
    };

    let mut oracle = DecyOracle::new(config)
        .context("Failed to initialize oracle")?;

    // Initial transpilation
    let mut rust_code = decy_core::transpile(c_code)
        .context("Initial transpilation failed")?;

    let mut result = OracleTranspileResult {
        rust_code: rust_code.clone(),
        oracle_queries: 0,
        fixes_applied: 0,
        retries_used: 0,
        compilation_success: false,
        remaining_errors: Vec::new(),
    };

    // Oracle feedback loop
    for retry in 0..options.max_retries {
        // Check compilation
        match check_rust_compilation(&rust_code) {
            Ok(()) => {
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

/// Stub for non-oracle builds
#[cfg(not(feature = "oracle"))]
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
        remaining_errors: vec!["Oracle feature not enabled".into()],
    })
}

/// Check if Rust code compiles
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
#[cfg(feature = "oracle")]
fn apply_fix(rust_code: &str, diff: &str) -> Result<String, String> {
    // Simple line-based diff application
    // Format: "- old line\n+ new line"
    let mut result = rust_code.to_string();

    let lines: Vec<&str> = diff.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("- ") {
            let old = &line[2..];
            if i + 1 < lines.len() && lines[i + 1].starts_with("+ ") {
                let new = &lines[i + 1][2..];
                result = result.replace(old, new);
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_options_default() {
        let opts = OracleOptions::default();
        assert!(!opts.enabled);
        assert!(!opts.auto_fix);
        assert_eq!(opts.max_retries, 0);
    }

    #[test]
    fn test_oracle_options_new() {
        let opts = OracleOptions::new(true, Some(0.8), true);
        assert!(opts.enabled);
        assert!(opts.auto_fix);
        assert!((opts.threshold - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_should_use_oracle() {
        let opts = OracleOptions::new(true, None, false);
        assert!(opts.should_use_oracle());

        let opts = OracleOptions::new(false, None, false);
        assert!(!opts.should_use_oracle());
    }

    #[test]
    #[cfg(feature = "oracle")]
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
    #[cfg(feature = "oracle")]
    fn test_apply_fix() {
        let rust_code = "let x = value.clone();";
        let diff = "- value.clone()\n+ value.to_owned()";

        let result = apply_fix(rust_code, diff).unwrap();
        assert_eq!(result, "let x = value.to_owned();");
    }

    #[test]
    fn test_transpile_result_struct() {
        let result = OracleTranspileResult {
            rust_code: "fn main() {}".into(),
            oracle_queries: 5,
            fixes_applied: 2,
            retries_used: 1,
            compilation_success: true,
            remaining_errors: vec![],
        };

        assert!(result.compilation_success);
        assert_eq!(result.oracle_queries, 5);
        assert_eq!(result.fixes_applied, 2);
    }

    #[test]
    fn test_check_rust_compilation_valid() {
        let valid_code = "fn main() {}";
        let result = check_rust_compilation(valid_code);
        if let Err(ref e) = result {
            eprintln!("Compilation error: {}", e);
        }
        // Skip test if rustc is not available
        if result.as_ref().err().map(|e| e.contains("Failed to run rustc")).unwrap_or(false) {
            eprintln!("Skipping test: rustc not available");
            return;
        }
        assert!(result.is_ok(), "Expected compilation to succeed, got: {:?}", result);
    }

    #[test]
    fn test_check_rust_compilation_invalid() {
        let invalid_code = "fn main() { let x: i32 = \"not an int\"; }";
        let result = check_rust_compilation(invalid_code);
        assert!(result.is_err());
    }
}
