//! Void pointer analysis for generic type inference.
//!
//! Analyzes void* usage patterns to infer generic type parameters
//! for transformation to Rust generics.

use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Pattern type detected for void* usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoidPtrPattern {
    /// Generic data pointer (unknown pattern)
    Generic,
    /// Swap pattern: two void* + size parameter
    Swap,
    /// Compare pattern: two void* returning int
    Compare,
    /// Copy pattern: dest void* + src void* + size
    Copy,
}

/// Type constraint inferred from void* usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Must be readable (shared reference)
    Readable,
    /// Must be mutable (exclusive reference)
    Mutable,
    /// Must implement Copy (for memcpy-style operations)
    Copy,
    /// Must implement Clone
    Clone,
    /// Must implement PartialOrd (for compare operations)
    PartialOrd,
}

/// Analysis result for a void* parameter.
#[derive(Debug, Clone)]
pub struct VoidPtrInfo {
    /// Parameter name
    pub param_name: String,
    /// Detected pattern
    pub pattern: VoidPtrPattern,
    /// Inferred concrete types from casts
    pub inferred_types: Vec<HirType>,
    /// Type constraints from usage
    pub constraints: Vec<TypeConstraint>,
}

/// Analyzer for void* usage patterns.
pub struct VoidPtrAnalyzer;

impl VoidPtrAnalyzer {
    /// Create a new void pointer analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a function for void* usage patterns.
    pub fn analyze(&self, func: &HirFunction) -> Vec<VoidPtrInfo> {
        let mut results = Vec::new();

        // Find void* parameters
        let void_ptr_params: Vec<_> = func
            .parameters()
            .iter()
            .filter(|p| self.is_void_ptr(p.param_type()))
            .collect();

        if void_ptr_params.is_empty() {
            return results;
        }

        // Detect pattern based on function signature
        let pattern = self.detect_pattern(func, &void_ptr_params);

        // Analyze each void* parameter
        for param in void_ptr_params {
            let mut info = VoidPtrInfo {
                param_name: param.name().to_string(),
                pattern: pattern.clone(),
                inferred_types: Vec::new(),
                constraints: Vec::new(),
            };

            // Analyze body for type casts and usage
            self.analyze_body(func.body(), param.name(), &mut info);

            results.push(info);
        }

        results
    }

    fn is_void_ptr(&self, ty: &HirType) -> bool {
        matches!(ty, HirType::Pointer(inner) if matches!(inner.as_ref(), HirType::Void))
    }

    fn detect_pattern(
        &self,
        func: &HirFunction,
        void_params: &[&decy_hir::HirParameter],
    ) -> VoidPtrPattern {
        let param_count = void_params.len();
        let has_size_param = func
            .parameters()
            .iter()
            .any(|p| p.name().contains("size") || p.name() == "n" || p.name() == "len");
        let returns_int = matches!(func.return_type(), HirType::Int);

        // Swap pattern: two void* + size
        if param_count == 2 && has_size_param && func.name() == "swap" {
            return VoidPtrPattern::Swap;
        }

        // Compare pattern: two void* returning int
        if param_count == 2
            && returns_int
            && (func.name().contains("cmp") || func.name() == "compare")
        {
            return VoidPtrPattern::Compare;
        }

        // Copy pattern: dest + src + size
        if param_count == 2 && has_size_param {
            let names: Vec<&str> = void_params.iter().map(|p| p.name()).collect();
            if names.contains(&"dest") || names.contains(&"src") || func.name().contains("copy") {
                return VoidPtrPattern::Copy;
            }
        }

        VoidPtrPattern::Generic
    }

    fn analyze_body(&self, stmts: &[HirStatement], param_name: &str, info: &mut VoidPtrInfo) {
        for stmt in stmts {
            self.analyze_statement(stmt, param_name, info);
        }
    }

    fn analyze_statement(&self, stmt: &HirStatement, param_name: &str, info: &mut VoidPtrInfo) {
        match stmt {
            HirStatement::VariableDeclaration {
                initializer: Some(init),
                ..
            } => {
                self.analyze_expression(init, param_name, info);
            }
            HirStatement::DerefAssignment { target, value } => {
                // Write through void* - implies mutable constraint
                if self.expr_uses_param(target, param_name)
                    && !info.constraints.contains(&TypeConstraint::Mutable)
                {
                    info.constraints.push(TypeConstraint::Mutable);
                }
                self.analyze_expression(target, param_name, info);
                self.analyze_expression(value, param_name, info);
            }
            HirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.analyze_expression(condition, param_name, info);
                self.analyze_body(then_block, param_name, info);
                if let Some(else_stmts) = else_block {
                    self.analyze_body(else_stmts, param_name, info);
                }
            }
            HirStatement::While {
                condition, body, ..
            } => {
                self.analyze_expression(condition, param_name, info);
                self.analyze_body(body, param_name, info);
            }
            HirStatement::For { body, .. } => {
                self.analyze_body(body, param_name, info);
            }
            HirStatement::Expression(expr) => {
                self.analyze_expression(expr, param_name, info);
            }
            HirStatement::Return(Some(expr)) => {
                self.analyze_expression(expr, param_name, info);
            }
            _ => {}
        }
    }

    fn analyze_expression(&self, expr: &HirExpression, param_name: &str, info: &mut VoidPtrInfo) {
        match expr {
            HirExpression::Cast {
                expr: inner,
                target_type,
            } => {
                // Found a cast - extract the type
                if self.expr_uses_param(inner, param_name) {
                    if let HirType::Pointer(inner_type) = target_type {
                        if !info.inferred_types.contains(inner_type) {
                            info.inferred_types.push((**inner_type).clone());
                        }
                    }
                }
            }
            HirExpression::BinaryOp { left, right, .. } => {
                self.analyze_expression(left, param_name, info);
                self.analyze_expression(right, param_name, info);
            }
            HirExpression::Dereference(inner) => {
                self.analyze_expression(inner, param_name, info);
            }
            HirExpression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression(arg, param_name, info);
                }
            }
            _ => {}
        }
    }

    fn expr_uses_param(&self, expr: &HirExpression, param_name: &str) -> bool {
        match expr {
            HirExpression::Variable(name) => name == param_name,
            HirExpression::Cast { expr: inner, .. } => self.expr_uses_param(inner, param_name),
            HirExpression::Dereference(inner) => self.expr_uses_param(inner, param_name),
            _ => false,
        }
    }
}

impl Default for VoidPtrAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
