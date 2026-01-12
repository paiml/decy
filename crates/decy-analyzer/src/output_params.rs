//! Output parameter detection for C-to-Rust transformation.
//!
//! This module identifies C output parameters (pointer parameters written before being read)
//! and classifies them for transformation to idiomatic Rust return values.
//!
//! # Examples
//!
//! C code with output parameter:
//! ```c
//! int parse(const char* input, int* result) {
//!     *result = 42;
//!     return 0;  // 0 = success
//! }
//! ```
//!
//! This would be transformed to idiomatic Rust:
//! ```rust,no_run
//! fn parse(input: &str) -> Result<i32, std::io::Error> {
//!     Ok(42)
//! }
//! ```

use decy_hir::HirFunction;
use std::collections::HashMap;

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
    pub fn detect(&self, func: &HirFunction) -> Vec<OutputParameter> {
        let mut results = Vec::new();

        // Track reads and writes for each parameter
        let mut reads: HashMap<String, bool> = HashMap::new();
        let mut writes: HashMap<String, bool> = HashMap::new();

        // Initialize tracking for pointer parameters only
        for param in func.parameters() {
            if Self::is_pointer_type(param.param_type()) {
                reads.insert(param.name().to_string(), false);
                writes.insert(param.name().to_string(), false);
            }
        }

        // Analyze function body
        for stmt in func.body() {
            Self::analyze_statement_internal(stmt, &mut reads, &mut writes);
        }

        // Detect fallible functions (multiple return values, typically 0 for success, non-zero for error)
        let is_fallible = self.is_fallible_function(func);

        // Classify parameters
        for param in func.parameters() {
            let param_name = param.name();

            if !Self::is_pointer_type(param.param_type()) {
                continue;
            }

            let was_read = reads.get(param_name).copied().unwrap_or(false);
            let was_written = writes.get(param_name).copied().unwrap_or(false);

            // Output parameter: written but not read (or written before read)
            if was_written && !was_read {
                results.push(OutputParameter {
                    name: param_name.to_string(),
                    kind: ParameterKind::Output,
                    is_fallible,
                });
            }
        }

        results
    }

    /// Check if a type is a pointer type.
    fn is_pointer_type(ty: &decy_hir::HirType) -> bool {
        matches!(ty, decy_hir::HirType::Pointer(_))
    }

    /// Check if a function is fallible (returns error codes).
    ///
    /// Heuristics:
    /// - Return type is Int (common for error codes: 0 = success, -1/1 = error)
    /// - Void functions are never fallible
    fn is_fallible_function(&self, func: &HirFunction) -> bool {
        use decy_hir::HirType;

        // Void functions are not fallible
        if matches!(func.return_type(), HirType::Void) {
            return false;
        }

        // Int return type with output parameters is a strong signal for error codes
        // Common C pattern: int func(input, output*) where int is 0=success, -1=error
        matches!(func.return_type(), HirType::Int)
    }

    /// Analyze a statement to track parameter reads and writes.
    fn analyze_statement_internal(
        stmt: &decy_hir::HirStatement,
        reads: &mut HashMap<String, bool>,
        writes: &mut HashMap<String, bool>,
    ) {
        use decy_hir::{HirExpression, HirStatement};

        match stmt {
            // Track dereference assignments: *ptr = value
            HirStatement::DerefAssignment { target, value } => {
                // Check if target is a parameter (write)
                if let HirExpression::Variable(var_name) = target {
                    if writes.contains_key(var_name) {
                        // Mark as written only if not already read
                        if !reads.get(var_name).copied().unwrap_or(false) {
                            writes.insert(var_name.clone(), true);
                        }
                    }
                }

                // Check value expression for reads
                Self::analyze_expression_internal(value, reads);
            }

            // Variable declarations can read from parameters
            HirStatement::VariableDeclaration {
                initializer: Some(expr),
                ..
            } => {
                Self::analyze_expression_internal(expr, reads);
            }

            // Assignment can read from parameters
            HirStatement::Assignment { value, .. } => {
                Self::analyze_expression_internal(value, reads);
            }

            // Return statement can read from parameters
            HirStatement::Return(Some(expr)) => {
                Self::analyze_expression_internal(expr, reads);
            }

            // Control flow statements
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::analyze_expression_internal(condition, reads);
                for s in then_block {
                    Self::analyze_statement_internal(s, reads, writes);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        Self::analyze_statement_internal(s, reads, writes);
                    }
                }
            }

            HirStatement::While { condition, body } => {
                Self::analyze_expression_internal(condition, reads);
                for s in body {
                    Self::analyze_statement_internal(s, reads, writes);
                }
            }

            HirStatement::For {
                init,
                condition,
                increment,
                body,
            } => {
                // DECY-224: Handle multiple init statements
                for init_stmt in init {
                    Self::analyze_statement_internal(init_stmt, reads, writes);
                }
                Self::analyze_expression_internal(condition, reads);
                // DECY-224: Handle multiple increment statements
                for inc_stmt in increment {
                    Self::analyze_statement_internal(inc_stmt, reads, writes);
                }
                for s in body {
                    Self::analyze_statement_internal(s, reads, writes);
                }
            }

            HirStatement::Switch {
                condition,
                cases,
                default_case,
            } => {
                Self::analyze_expression_internal(condition, reads);
                for case in cases {
                    for s in &case.body {
                        Self::analyze_statement_internal(s, reads, writes);
                    }
                }
                if let Some(default_stmts) = default_case {
                    for s in default_stmts {
                        Self::analyze_statement_internal(s, reads, writes);
                    }
                }
            }

            HirStatement::Expression(expr) => {
                Self::analyze_expression_internal(expr, reads);
            }

            _ => {}
        }
    }

    /// Analyze an expression to track parameter reads.
    fn analyze_expression_internal(
        expr: &decy_hir::HirExpression,
        reads: &mut HashMap<String, bool>,
    ) {
        use decy_hir::HirExpression;

        match expr {
            // Dereferencing a parameter is a read
            HirExpression::Dereference(inner) => {
                if let HirExpression::Variable(var_name) = inner.as_ref() {
                    if reads.contains_key(var_name) {
                        reads.insert(var_name.clone(), true);
                    }
                }
            }

            // Binary operations
            HirExpression::BinaryOp { left, right, .. } => {
                Self::analyze_expression_internal(left, reads);
                Self::analyze_expression_internal(right, reads);
            }

            // Unary operations
            HirExpression::UnaryOp { operand, .. } => {
                Self::analyze_expression_internal(operand, reads);
            }

            // Function calls
            HirExpression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    Self::analyze_expression_internal(arg, reads);
                }
            }

            // Field access
            HirExpression::FieldAccess { object, .. }
            | HirExpression::PointerFieldAccess {
                pointer: object, ..
            } => {
                Self::analyze_expression_internal(object, reads);
            }

            // Array indexing
            HirExpression::ArrayIndex { array, index }
            | HirExpression::SliceIndex {
                slice: array,
                index,
                ..
            } => {
                Self::analyze_expression_internal(array, reads);
                Self::analyze_expression_internal(index, reads);
            }

            _ => {}
        }
    }
}

impl Default for OutputParamDetector {
    fn default() -> Self {
        Self::new()
    }
}
