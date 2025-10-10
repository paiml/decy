//! Rust code generation from HIR with minimal unsafe blocks.
//!
//! Generates idiomatic Rust code with <5 unsafe blocks per 1000 LOC.
//!
//! # Examples
//!
//! ```
//! use decy_codegen::CodeGenerator;
//! use decy_hir::{HirFunction, HirType, HirParameter};
//!
//! let func = HirFunction::new(
//!     "add".to_string(),
//!     HirType::Int,
//!     vec![
//!         HirParameter::new("a".to_string(), HirType::Int),
//!         HirParameter::new("b".to_string(), HirType::Int),
//!     ],
//! );
//!
//! let codegen = CodeGenerator::new();
//! let code = codegen.generate_function(&func);
//!
//! assert!(code.contains("fn add"));
//! assert!(code.contains("a: i32"));
//! assert!(code.contains("b: i32"));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod box_transform;
pub mod test_generator;

use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};
use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};

/// Code generator for converting HIR to Rust source code.
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    box_transformer: box_transform::BoxTransformer,
}

impl CodeGenerator {
    /// Create a new code generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    ///
    /// let codegen = CodeGenerator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            box_transformer: box_transform::BoxTransformer::new(),
        }
    }

    /// Get the Box transformer.
    pub fn box_transformer(&self) -> &box_transform::BoxTransformer {
        &self.box_transformer
    }

    /// Map HIR type to Rust type string.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::HirType;
    ///
    /// assert_eq!(CodeGenerator::map_type(&HirType::Int), "i32");
    /// assert_eq!(CodeGenerator::map_type(&HirType::Float), "f32");
    /// assert_eq!(CodeGenerator::map_type(&HirType::Box(Box::new(HirType::Int))), "Box<i32>");
    /// ```
    pub fn map_type(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "i32".to_string(),
            HirType::Float => "f32".to_string(),
            HirType::Double => "f64".to_string(),
            HirType::Char => "u8".to_string(),
            HirType::Pointer(inner) => {
                format!("*mut {}", Self::map_type(inner))
            }
            HirType::Box(inner) => {
                format!("Box<{}>", Self::map_type(inner))
            }
            HirType::Vec(inner) => {
                format!("Vec<{}>", Self::map_type(inner))
            }
            HirType::Reference { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", Self::map_type(inner))
                } else {
                    format!("&{}", Self::map_type(inner))
                }
            }
        }
    }

    /// Generate code for an expression.
    #[allow(clippy::only_used_in_recursion)]
    pub fn generate_expression(&self, expr: &HirExpression) -> String {
        match expr {
            HirExpression::IntLiteral(val) => val.to_string(),
            HirExpression::Variable(name) => name.clone(),
            HirExpression::BinaryOp { op, left, right } => {
                let left_code = self.generate_expression(left);
                let right_code = self.generate_expression(right);
                let op_str = Self::binary_operator_to_string(op);

                // Add parentheses for nested binary operations
                let left_str = if matches!(**left, HirExpression::BinaryOp { .. }) {
                    format!("({})", left_code)
                } else {
                    left_code
                };

                let right_str = if matches!(**right, HirExpression::BinaryOp { .. }) {
                    format!("({})", right_code)
                } else {
                    right_code
                };

                format!("{} {} {}", left_str, op_str, right_str)
            }
            HirExpression::Dereference(inner) => {
                let inner_code = self.generate_expression(inner);
                format!("*{}", inner_code)
            }
            HirExpression::AddressOf(inner) => {
                let inner_code = self.generate_expression(inner);
                // Add parentheses for non-trivial expressions
                if matches!(**inner, HirExpression::Dereference(_)) {
                    format!("&({})", inner_code)
                } else {
                    format!("&{}", inner_code)
                }
            }
            HirExpression::FunctionCall {
                function,
                arguments,
            } => {
                let args: Vec<String> = arguments
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect();
                format!("{}({})", function, args.join(", "))
            }
        }
    }

    /// Convert binary operator to string.
    fn binary_operator_to_string(op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterEqual => ">=",
        }
    }

    /// Get default value for a type (for uninitialized variables).
    fn default_value_for_type(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "0".to_string(),
            HirType::Float => "0.0".to_string(),
            HirType::Double => "0.0".to_string(),
            HirType::Char => "0".to_string(),
            HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
            HirType::Box(inner) => {
                // Box types should not use default values, they should be initialized with Box::new
                // This is just a fallback
                format!("Box::new({})", Self::default_value_for_type(inner))
            }
            HirType::Vec(_) => {
                // Vec types default to empty Vec
                "Vec::new()".to_string()
            }
            HirType::Reference { .. } => {
                // References cannot have default values - they must always be initialized
                // This should never be reached in valid code
                panic!("References must be initialized and cannot have default values")
            }
        }
    }

    /// Generate code for a statement.
    pub fn generate_statement(&self, stmt: &HirStatement) -> String {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                let mut code = format!("let mut {}: {}", name, Self::map_type(var_type));
                if let Some(init_expr) = initializer {
                    code.push_str(&format!(" = {};", self.generate_expression(init_expr)));
                } else {
                    // Provide default value for uninitialized variables
                    code.push_str(&format!(" = {};", Self::default_value_for_type(var_type)));
                }
                code
            }
            HirStatement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    format!("return {};", self.generate_expression(expr))
                } else {
                    "return;".to_string()
                }
            }
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut code = String::new();

                // Generate if condition
                code.push_str(&format!("if {} {{\n", self.generate_expression(condition)));

                // Generate then block
                for stmt in then_block {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement(stmt));
                    code.push('\n');
                }

                // Generate else block if present
                if let Some(else_stmts) = else_block {
                    code.push_str("} else {\n");
                    for stmt in else_stmts {
                        code.push_str("    ");
                        code.push_str(&self.generate_statement(stmt));
                        code.push('\n');
                    }
                }

                code.push('}');
                code
            }
            HirStatement::While { condition, body } => {
                let mut code = String::new();

                // Generate while condition
                code.push_str(&format!(
                    "while {} {{\n",
                    self.generate_expression(condition)
                ));

                // Generate loop body
                for stmt in body {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement(stmt));
                    code.push('\n');
                }

                code.push('}');
                code
            }
            HirStatement::Break => "break;".to_string(),
            HirStatement::Continue => "continue;".to_string(),
            HirStatement::Assignment { target, value } => {
                format!("{} = {};", target, self.generate_expression(value))
            }
        }
    }

    /// Generate a function signature from HIR.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType};
    ///
    /// let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    /// let codegen = CodeGenerator::new();
    /// let sig = codegen.generate_signature(&func);
    ///
    /// assert_eq!(sig, "fn test()");
    /// ```
    pub fn generate_signature(&self, func: &HirFunction) -> String {
        let mut sig = format!("fn {}", func.name());

        // Generate parameters
        sig.push('(');
        let params: Vec<String> = func
            .parameters()
            .iter()
            .map(|p| format!("{}: {}", p.name(), Self::map_type(p.param_type())))
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');

        // Generate return type (skip for void)
        if !matches!(func.return_type(), HirType::Void) {
            sig.push_str(&format!(" -> {}", Self::map_type(func.return_type())));
        }

        sig
    }

    /// Generate a function signature with lifetime annotations.
    ///
    /// Takes an `AnnotatedSignature` with lifetime information and generates
    /// the complete Rust function signature including lifetime parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedParameter, AnnotatedType, LifetimeParam};
    /// use decy_hir::HirType;
    ///
    /// let sig = AnnotatedSignature {
    ///     name: "get_first".to_string(),
    ///     lifetimes: vec![LifetimeParam::standard(0)], // 'a
    ///     parameters: vec![
    ///         AnnotatedParameter {
    ///             name: "items".to_string(),
    ///             param_type: AnnotatedType::Reference {
    ///                 inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///                 mutable: false,
    ///                 lifetime: Some(LifetimeParam::standard(0)),
    ///             },
    ///         },
    ///     ],
    ///     return_type: AnnotatedType::Reference {
    ///         inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///         mutable: false,
    ///         lifetime: Some(LifetimeParam::standard(0)),
    ///     },
    /// };
    ///
    /// let codegen = CodeGenerator::new();
    /// let rust_sig = codegen.generate_annotated_signature(&sig);
    ///
    /// assert!(rust_sig.contains("<'a>"));
    /// assert!(rust_sig.contains("&'a i32"));
    /// ```
    pub fn generate_annotated_signature(&self, sig: &AnnotatedSignature) -> String {
        let mut result = format!("fn {}", sig.name);

        // Add lifetime parameters if present
        if !sig.lifetimes.is_empty() {
            let lifetime_params: Vec<String> =
                sig.lifetimes.iter().map(|lt| lt.name.clone()).collect();
            result.push_str(&format!("<{}>", lifetime_params.join(", ")));
        }

        // Add function parameters
        result.push('(');
        let params: Vec<String> = sig
            .parameters
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    p.name,
                    self.annotated_type_to_string(&p.param_type)
                )
            })
            .collect();
        result.push_str(&params.join(", "));
        result.push(')');

        // Add return type if not void
        let return_type_str = self.annotated_type_to_string(&sig.return_type);
        if return_type_str != "()" {
            result.push_str(&format!(" -> {}", return_type_str));
        }

        result
    }

    /// Convert an `AnnotatedType` to Rust type string with lifetime annotations.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_ownership::lifetime_gen::{AnnotatedType, LifetimeParam};
    /// use decy_hir::HirType;
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Simple type
    /// let simple = AnnotatedType::Simple(HirType::Int);
    /// assert_eq!(codegen.annotated_type_to_string(&simple), "i32");
    ///
    /// // Reference with lifetime
    /// let ref_type = AnnotatedType::Reference {
    ///     inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///     mutable: false,
    ///     lifetime: Some(LifetimeParam::standard(0)),
    /// };
    /// assert_eq!(codegen.annotated_type_to_string(&ref_type), "&'a i32");
    /// ```
    #[allow(clippy::only_used_in_recursion)]
    pub fn annotated_type_to_string(&self, annotated_type: &AnnotatedType) -> String {
        match annotated_type {
            AnnotatedType::Simple(hir_type) => Self::map_type(hir_type),
            AnnotatedType::Reference {
                inner,
                mutable,
                lifetime,
            } => {
                let mut result = String::from("&");

                // Add lifetime if present
                if let Some(lt) = lifetime {
                    result.push_str(&lt.name);
                    result.push(' ');
                }

                // Add mutability
                if *mutable {
                    result.push_str("mut ");
                }

                // Add inner type
                result.push_str(&self.annotated_type_to_string(inner));

                result
            }
        }
    }

    /// Generate a default return statement for a type.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::HirType;
    ///
    /// let codegen = CodeGenerator::new();
    /// assert!(codegen.generate_return(&HirType::Int).contains("return 0"));
    /// ```
    pub fn generate_return(&self, return_type: &HirType) -> String {
        match return_type {
            HirType::Void => String::new(),
            HirType::Int => "    return 0;".to_string(),
            HirType::Float => "    return 0.0;".to_string(),
            HirType::Double => "    return 0.0;".to_string(),
            HirType::Char => "    return 0;".to_string(),
            HirType::Pointer(_) => "    return std::ptr::null_mut();".to_string(),
            HirType::Box(inner) => {
                format!(
                    "    return Box::new({});",
                    Self::default_value_for_type(inner)
                )
            }
            HirType::Vec(_) => "    return Vec::new();".to_string(),
            HirType::Reference { .. } => {
                // References in return position need concrete values from parameters
                // This should be handled by lifetime-annotated code generation
                // using generate_function_with_lifetimes() instead
                String::new()
            }
        }
    }

    /// Generate a complete function from HIR.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirParameter};
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
    /// let codegen = CodeGenerator::new();
    /// let code = codegen.generate_function(&func);
    ///
    /// assert!(code.contains("fn add(a: i32, b: i32) -> i32"));
    /// assert!(code.contains("{"));
    /// assert!(code.contains("}"));
    /// ```
    pub fn generate_function(&self, func: &HirFunction) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements
            for stmt in func.body() {
                code.push_str("    ");
                code.push_str(&self.generate_statement(stmt));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a complete function from HIR with lifetime annotations.
    ///
    /// Takes both the HIR function and its annotated signature to generate
    /// Rust code with proper lifetime annotations.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirParameter};
    /// use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedParameter, AnnotatedType, LifetimeParam};
    ///
    /// let func = HirFunction::new(
    ///     "identity".to_string(),
    ///     HirType::Reference {
    ///         inner: Box::new(HirType::Int),
    ///         mutable: false,
    ///     },
    ///     vec![
    ///         HirParameter::new("x".to_string(), HirType::Reference {
    ///             inner: Box::new(HirType::Int),
    ///             mutable: false,
    ///         }),
    ///     ],
    /// );
    ///
    /// let sig = AnnotatedSignature {
    ///     name: "identity".to_string(),
    ///     lifetimes: vec![LifetimeParam::standard(0)],
    ///     parameters: vec![
    ///         AnnotatedParameter {
    ///             name: "x".to_string(),
    ///             param_type: AnnotatedType::Reference {
    ///                 inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///                 mutable: false,
    ///                 lifetime: Some(LifetimeParam::standard(0)),
    ///             },
    ///         },
    ///     ],
    ///     return_type: AnnotatedType::Reference {
    ///         inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///         mutable: false,
    ///         lifetime: Some(LifetimeParam::standard(0)),
    ///     },
    /// };
    ///
    /// let codegen = CodeGenerator::new();
    /// let code = codegen.generate_function_with_lifetimes(&func, &sig);
    ///
    /// assert!(code.contains("<'a>"));
    /// assert!(code.contains("&'a i32"));
    /// ```
    pub fn generate_function_with_lifetimes(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
    ) -> String {
        let mut code = String::new();

        // Generate signature with lifetimes
        code.push_str(&self.generate_annotated_signature(sig));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements
            for stmt in func.body() {
                code.push_str("    ");
                code.push_str(&self.generate_statement(stmt));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a function with Box transformations applied.
    ///
    /// This method analyzes the function for malloc/free patterns and
    /// transforms them into safe `Box::new()` expressions.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirStatement, HirExpression};
    /// use decy_analyzer::patterns::PatternDetector;
    ///
    /// let func = HirFunction::new_with_body(
    ///     "test".to_string(),
    ///     HirType::Void,
    ///     vec![],
    ///     vec![
    ///         HirStatement::VariableDeclaration {
    ///             name: "ptr".to_string(),
    ///             var_type: HirType::Pointer(Box::new(HirType::Int)),
    ///             initializer: Some(HirExpression::FunctionCall {
    ///                 function: "malloc".to_string(),
    ///                 arguments: vec![HirExpression::IntLiteral(100)],
    ///             }),
    ///         },
    ///     ],
    /// );
    ///
    /// let codegen = CodeGenerator::new();
    /// let detector = PatternDetector::new();
    /// let candidates = detector.find_box_candidates(&func);
    /// let code = codegen.generate_function_with_box_transform(&func, &candidates);
    ///
    /// assert!(code.contains("Box::new"));
    /// ```
    pub fn generate_function_with_box_transform(
        &self,
        func: &HirFunction,
        candidates: &[decy_analyzer::patterns::BoxCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with Box transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check if this statement should be transformed
                let transformed_stmt =
                    if let Some(candidate) = candidates.iter().find(|c| c.malloc_index == idx) {
                        self.box_transformer.transform_statement(stmt, candidate)
                    } else {
                        stmt.clone()
                    };

                code.push_str("    ");
                code.push_str(&self.generate_statement(&transformed_stmt));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a function with Vec transformations applied.
    ///
    /// This method analyzes the function for malloc(n * sizeof(T)) patterns and
    /// transforms them into safe `Vec::with_capacity(n)` expressions.
    pub fn generate_function_with_vec_transform(
        &self,
        func: &HirFunction,
        candidates: &[decy_analyzer::patterns::VecCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with Vec transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check if this statement should be transformed
                let transformed_stmt =
                    if let Some(candidate) = candidates.iter().find(|c| c.malloc_index == idx) {
                        self.transform_vec_statement(stmt, candidate)
                    } else {
                        stmt.clone()
                    };

                code.push_str("    ");
                code.push_str(&self.generate_statement(&transformed_stmt));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Transform a statement to use Vec instead of malloc for array patterns.
    fn transform_vec_statement(
        &self,
        stmt: &HirStatement,
        candidate: &decy_analyzer::patterns::VecCandidate,
    ) -> HirStatement {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer: _,
            } => {
                // Get the element type from the pointer
                let element_type = if let HirType::Pointer(inner) = var_type {
                    (**inner).clone()
                } else {
                    // Fallback: keep original type
                    return stmt.clone();
                };

                // Transform type to Vec
                let vec_type = HirType::Vec(Box::new(element_type));

                // Transform initializer: malloc(n * sizeof(T)) → Vec::with_capacity(n)
                let vec_initializer = if let Some(capacity_expr) = &candidate.capacity_expr {
                    Some(HirExpression::FunctionCall {
                        function: "Vec::with_capacity".to_string(),
                        arguments: vec![capacity_expr.clone()],
                    })
                } else {
                    // No capacity expression - use Vec::new()
                    Some(HirExpression::FunctionCall {
                        function: "Vec::new".to_string(),
                        arguments: vec![],
                    })
                };

                HirStatement::VariableDeclaration {
                    name: name.clone(),
                    var_type: vec_type,
                    initializer: vec_initializer,
                }
            }
            HirStatement::Assignment {
                target: _,
                value: _,
            } => {
                // Similar transformation for assignments
                // For now, keep the original statement
                // Future: handle ptr = malloc(n * sizeof(T)) assignments
                stmt.clone()
            }
            _ => stmt.clone(),
        }
    }

    /// Generate a function with both Box and Vec transformations applied.
    ///
    /// This method combines both Box and Vec transformations,
    /// applying them to their respective patterns.
    pub fn generate_function_with_box_and_vec_transform(
        &self,
        func: &HirFunction,
        box_candidates: &[decy_analyzer::patterns::BoxCandidate],
        vec_candidates: &[decy_analyzer::patterns::VecCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with both transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check Vec candidates first (more specific pattern)
                let transformed_stmt = if let Some(vec_candidate) =
                    vec_candidates.iter().find(|c| c.malloc_index == idx)
                {
                    self.transform_vec_statement(stmt, vec_candidate)
                } else if let Some(box_candidate) =
                    box_candidates.iter().find(|c| c.malloc_index == idx)
                {
                    self.box_transformer
                        .transform_statement(stmt, box_candidate)
                } else {
                    stmt.clone()
                };

                code.push_str("    ");
                code.push_str(&self.generate_statement(&transformed_stmt));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "codegen_tests.rs"]
mod codegen_tests;

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;

#[cfg(test)]
#[path = "vec_property_tests.rs"]
mod vec_property_tests;
