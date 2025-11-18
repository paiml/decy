//! Output parameter detection for C-to-Rust transformation.
//!
//! This module identifies C output parameters (pointer parameters written before being read)
//! and classifies them for transformation to idiomatic Rust return values.
//!
//! # Examples
//!
//! ```c
//! // C code with output parameter:
//! int parse(const char* input, int* result) {
//!     *result = 42;
//!     return 0;  // 0 = success
//! }
//! ```
//!
//! This would be transformed to:
//!
//! ```rust
//! fn parse(input: &str) -> Result<i32, ParseError> {
//!     Ok(42)
//! }
//! ```

use decy_hir::HirFunction;

/// Represents a detected output parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputParameter {
    /// Parameter name
    pub name: String,
    /// Parameter kind (output vs input-output)
    pub kind: ParameterKind,
    /// Whether the function is fallible (returns error codes)
    pub is_fallible: bool,
}

/// Classification of parameter usage patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    /// Pure output parameter (written before read)
    Output,
    /// Input-output parameter (read before or during write)
    InputOutput,
}

/// Detector for output parameters in C functions.
#[derive(Debug, Clone)]
pub struct OutputParamDetector;

impl OutputParamDetector {
    /// Create a new output parameter detector.
    pub fn new() -> Self {
        Self
    }

    /// Detect output parameters in a function.
    ///
    /// # Arguments
    ///
    /// * `func` - The HIR function to analyze
    ///
    /// # Returns
    ///
    /// A vector of detected output parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_analyzer::output_params::OutputParamDetector;
    /// use decy_hir::HirFunction;
    ///
    /// let detector = OutputParamDetector::new();
    /// // let func = ...; // Create HIR function
    /// // let params = detector.detect(&func);
    /// ```
    pub fn detect(&self, _func: &HirFunction) -> Vec<OutputParameter> {
        // TODO: Implement detection logic
        // This is the RED phase - tests will fail
        vec![]
    }
}

impl Default for OutputParamDetector {
    fn default() -> Self {
        Self::new()
    }
}
