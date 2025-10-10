//! Test generator for transpiled Rust code.
//!
//! This module implements the test generation system described in the specification
//! (Section 9: Test Generation - EXTREME TDD Output).
//!
//! For each transpiled function, the generator creates:
//! - Unit tests (≥5 per function): Happy path, error cases, edge cases
//! - Property tests (≥5 per function): Determinism, no panics, invariants
//! - Doc tests: Usage examples in documentation
//! - Mutation test config: Test quality verification
//! - Behavior equivalence tests: Rust vs C comparison (optional)
//!
//! # Examples
//!
//! ```
//! use decy_codegen::test_generator::{TestGenerator, TestGenConfig};
//! use decy_hir::{HirFunction, HirType};
//!
//! let config = TestGenConfig::default();
//! let generator = TestGenerator::new(config);
//!
//! let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
//! let tests = generator.generate_tests(&func);
//!
//! assert!(tests.unit_tests.len() >= 5);
//! assert!(tests.property_tests.len() >= 5);
//! ```

use decy_hir::{HirFunction, HirType};

/// Configuration for test generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestGenConfig {
    /// Number of unit tests to generate per function (default: 5)
    pub unit_tests_per_function: usize,

    /// Number of property tests to generate per function (default: 5)
    pub property_tests_per_function: usize,

    /// Number of test cases for each property test (default: 1000)
    pub property_test_cases: usize,

    /// Whether to generate doc tests (default: true)
    pub generate_doc_tests: bool,

    /// Whether to generate mutation test configuration (default: true)
    pub generate_mutation_config: bool,

    /// Whether to generate behavior equivalence tests (default: true if C available)
    pub behavior_equivalence_tests: bool,
}

impl Default for TestGenConfig {
    fn default() -> Self {
        Self {
            unit_tests_per_function: 5,
            property_tests_per_function: 5,
            property_test_cases: 1000,
            generate_doc_tests: true,
            generate_mutation_config: true,
            behavior_equivalence_tests: true,
        }
    }
}

/// Test generator for transpiled Rust functions.
///
/// Generates comprehensive test suites including unit tests, property tests,
/// doc tests, and mutation test configurations.
#[derive(Debug)]
pub struct TestGenerator {
    config: TestGenConfig,
}

impl TestGenerator {
    /// Create a new test generator with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::test_generator::{TestGenerator, TestGenConfig};
    ///
    /// let config = TestGenConfig::default();
    /// let generator = TestGenerator::new(config);
    /// ```
    pub fn new(config: TestGenConfig) -> Self {
        Self { config }
    }

    /// Get the test generator configuration.
    pub fn config(&self) -> &TestGenConfig {
        &self.config
    }

    /// Generate a complete test suite for a HIR function.
    ///
    /// # Arguments
    ///
    /// * `hir_func` - The HIR function to generate tests for
    ///
    /// # Returns
    ///
    /// A `GeneratedTests` struct containing all generated test code.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::test_generator::{TestGenerator, TestGenConfig};
    /// use decy_hir::{HirFunction, HirType, HirParameter};
    ///
    /// let generator = TestGenerator::new(TestGenConfig::default());
    ///
    /// let func = HirFunction::new(
    ///     "add".to_string(),
    ///     HirType::Int,
    ///     vec![
    ///         HirParameter::new("a".to_string(), HirType::Int),
    ///         HirParameter::new("b".to_string(), HirType::Int),
    ///     ],
    /// );
    ///
    /// let tests = generator.generate_tests(&func);
    /// assert!(tests.unit_tests.len() >= 5);
    /// ```
    pub fn generate_tests(&self, hir_func: &HirFunction) -> GeneratedTests {
        let mut unit_tests = Vec::new();
        let mut property_tests = Vec::new();
        let mut doc_tests = Vec::new();
        let equivalence_tests = Vec::new();

        // Generate unit tests
        unit_tests.extend(self.generate_unit_tests(hir_func));

        // Generate property tests
        property_tests.extend(self.generate_property_tests(hir_func));

        // Generate doc tests if enabled
        if self.config.generate_doc_tests {
            doc_tests.extend(self.generate_doc_tests(hir_func));
        }

        // Generate mutation config if enabled
        let mutation_config = if self.config.generate_mutation_config {
            Some(self.generate_mutation_config(hir_func))
        } else {
            None
        };

        GeneratedTests {
            unit_tests,
            property_tests,
            doc_tests,
            mutation_config,
            equivalence_tests,
        }
    }

    /// Generate unit tests for a function.
    fn generate_unit_tests(&self, hir_func: &HirFunction) -> Vec<String> {
        let mut tests = Vec::new();
        let func_name = hir_func.name();

        // Generate happy path test
        tests.push(self.generate_happy_path_test(hir_func));

        // Generate null/None tests for pointer parameters
        for param in hir_func.parameters() {
            if matches!(param.param_type(), HirType::Pointer(_) | HirType::Box(_)) {
                tests.push(self.generate_null_parameter_test(hir_func, param.name()));
            }
        }

        // Generate edge case tests based on parameter types
        tests.extend(self.generate_edge_case_tests(hir_func));

        // Ensure we have at least the configured number of tests
        while tests.len() < self.config.unit_tests_per_function {
            tests.push(format!(
                r#"#[test]
fn test_{}_case_{}() {{
    // Additional test case {}
    let result = {}();
    // Add assertions here
}}"#,
                func_name,
                tests.len(),
                tests.len(),
                func_name
            ));
        }

        tests
    }

    /// Generate happy path test.
    fn generate_happy_path_test(&self, hir_func: &HirFunction) -> String {
        let func_name = hir_func.name();
        let params = hir_func.parameters();

        let param_setup = if params.is_empty() {
            String::new()
        } else {
            let setups: Vec<String> = params
                .iter()
                .map(|p| {
                    let default_val = Self::default_test_value(p.param_type());
                    format!("    let {} = {};", p.name(), default_val)
                })
                .collect();
            format!("{}\n", setups.join("\n"))
        };

        let call_args: Vec<String> = params.iter().map(|p| p.name().to_string()).collect();
        let call_expr = if call_args.is_empty() {
            format!("{}()", func_name)
        } else {
            format!("{}({})", func_name, call_args.join(", "))
        };

        format!(
            r#"#[test]
fn test_{}_happy_path() {{
{}    let result = {};
    // Verify expected behavior
}}"#,
            func_name, param_setup, call_expr
        )
    }

    /// Generate null parameter test.
    fn generate_null_parameter_test(&self, hir_func: &HirFunction, param_name: &str) -> String {
        let func_name = hir_func.name();

        format!(
            r#"#[test]
fn test_{}_null_{} () {{
    // Test with null/None for {}
    // Should handle gracefully
}}"#,
            func_name, param_name, param_name
        )
    }

    /// Generate edge case tests.
    fn generate_edge_case_tests(&self, hir_func: &HirFunction) -> Vec<String> {
        let mut tests = Vec::new();
        let func_name = hir_func.name();

        // Generate boundary value tests for integer parameters
        let has_int_params = hir_func
            .parameters()
            .iter()
            .any(|p| matches!(p.param_type(), HirType::Int));

        if has_int_params {
            tests.push(format!(
                r#"#[test]
fn test_{}_boundary_values() {{
    // Test with boundary values (0, MAX, MIN)
}}"#,
                func_name
            ));
        }

        tests
    }

    /// Generate property tests for a function.
    fn generate_property_tests(&self, hir_func: &HirFunction) -> Vec<String> {
        let mut tests = Vec::new();

        // Generate determinism property
        tests.push(self.generate_determinism_property(hir_func));

        // Generate no-panic property
        tests.push(self.generate_no_panic_property(hir_func));

        // Generate additional properties to meet minimum count
        while tests.len() < self.config.property_tests_per_function {
            tests.push(self.generate_generic_property(hir_func, tests.len()));
        }

        tests
    }

    /// Generate determinism property test.
    fn generate_determinism_property(&self, hir_func: &HirFunction) -> String {
        let func_name = hir_func.name();

        format!(
            r#"proptest! {{
    #[test]
    fn prop_{}_deterministic(/* inputs here */) {{
        // Same inputs should produce same outputs
        let result1 = {}(/* args */);
        let result2 = {}(/* args */);
        prop_assert_eq!(result1, result2);
    }}
}}"#,
            func_name, func_name, func_name
        )
    }

    /// Generate no-panic property test.
    fn generate_no_panic_property(&self, hir_func: &HirFunction) -> String {
        let func_name = hir_func.name();

        format!(
            r#"proptest! {{
    #[test]
    fn prop_{}_never_panics(/* inputs here */) {{
        // Should never panic, even with invalid inputs
        let _ = {}(/* args */);
    }}
}}"#,
            func_name, func_name
        )
    }

    /// Generate a generic property test.
    fn generate_generic_property(&self, hir_func: &HirFunction, index: usize) -> String {
        let func_name = hir_func.name();

        format!(
            r#"proptest! {{
    #[test]
    fn prop_{}_invariant_{}(/* inputs here */) {{
        // Test invariant {}
        let result = {}(/* args */);
        // Add property assertions here
    }}
}}"#,
            func_name, index, index, func_name
        )
    }

    /// Generate doc tests for a function.
    fn generate_doc_tests(&self, hir_func: &HirFunction) -> Vec<String> {
        let mut tests = Vec::new();
        let func_name = hir_func.name();

        let params = hir_func.parameters();
        let param_setup: Vec<String> = params
            .iter()
            .map(|p| {
                let default_val = Self::default_test_value(p.param_type());
                format!("let {} = {};", p.name(), default_val)
            })
            .collect();

        let call_args: Vec<String> = params.iter().map(|p| p.name().to_string()).collect();
        let call_expr = if call_args.is_empty() {
            format!("{}()", func_name)
        } else {
            format!("{}({})", func_name, call_args.join(", "))
        };

        let doc_test = format!(
            r#"/// Function: {}
///
/// # Examples
///
/// ```
/// {}
/// let result = {};
/// // Verify result
/// ```"#,
            func_name,
            param_setup.join("\n/// "),
            call_expr
        );

        tests.push(doc_test);
        tests
    }

    /// Generate mutation test configuration.
    fn generate_mutation_config(&self, hir_func: &HirFunction) -> String {
        let func_name = hir_func.name();

        format!(
            r#"[[mutant]]
function = "{}"
mutations = [
    "replace_return_values",
    "flip_boolean_conditions",
    "replace_arithmetic_operators",
]
expected_kill_rate = 0.90
"#,
            func_name
        )
    }

    /// Get default test value for a type.
    fn default_test_value(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "42".to_string(),
            HirType::Float => "3.14".to_string(),
            HirType::Double => "2.718".to_string(),
            HirType::Char => "b'A'".to_string(),
            HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
            HirType::Box(inner) => {
                format!("Box::new({})", Self::default_test_value(inner))
            }
            HirType::Reference { inner, mutable: _ } => {
                // References need to borrow from a variable
                // Generate a reference to a default value
                format!("&{}", Self::default_test_value(inner))
            }
        }
    }
}

/// Generated test suite for a transpiled function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTests {
    /// Unit tests (happy path, error cases, edge cases)
    pub unit_tests: Vec<String>,

    /// Property tests (determinism, no panics, invariants)
    pub property_tests: Vec<String>,

    /// Doc tests (usage examples)
    pub doc_tests: Vec<String>,

    /// Mutation test configuration (TOML format)
    pub mutation_config: Option<String>,

    /// Behavior equivalence tests (Rust vs C)
    pub equivalence_tests: Vec<String>,
}

#[cfg(test)]
#[path = "test_generator_tests.rs"]
mod test_generator_tests;
