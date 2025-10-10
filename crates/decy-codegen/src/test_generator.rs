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

use decy_hir::HirFunction;

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
    pub fn generate_tests(&self, _hir_func: &HirFunction) -> GeneratedTests {
        // Stub implementation - will be implemented in GREEN phase
        GeneratedTests {
            unit_tests: Vec::new(),
            property_tests: Vec::new(),
            doc_tests: Vec::new(),
            mutation_config: None,
            equivalence_tests: Vec::new(),
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
