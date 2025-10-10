//! High-level Intermediate Representation for C-to-Rust transpilation.
//!
//! The HIR is a Rust-oriented representation that bridges C AST and Rust code generation.
//!
//! # Examples
//!
//! ```
//! use decy_hir::{HirFunction, HirType, HirParameter};
//!
//! // Create a simple HIR function
//! let func = HirFunction::new(
//!     "main".to_string(),
//!     HirType::Int,
//!     vec![],
//! );
//!
//! assert_eq!(func.name(), "main");
//! assert_eq!(func.return_type(), &HirType::Int);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

/// Represents a C type in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirType {
    /// void type
    Void,
    /// int type (maps to i32 in Rust)
    Int,
    /// float type (maps to f32 in Rust)
    Float,
    /// double type (maps to f64 in Rust)
    Double,
    /// char type (maps to u8 in Rust)
    Char,
    /// Pointer to another type
    Pointer(Box<HirType>),
}

impl HirType {
    /// Convert from parser AST type to HIR type.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_hir::HirType;
    /// use decy_parser::parser::Type;
    ///
    /// let hir_type = HirType::from_ast_type(&Type::Int);
    /// assert_eq!(hir_type, HirType::Int);
    /// ```
    pub fn from_ast_type(ast_type: &decy_parser::parser::Type) -> Self {
        use decy_parser::parser::Type;
        match ast_type {
            Type::Void => HirType::Void,
            Type::Int => HirType::Int,
            Type::Float => HirType::Float,
            Type::Double => HirType::Double,
            Type::Char => HirType::Char,
            Type::Pointer(inner) => HirType::Pointer(Box::new(HirType::from_ast_type(inner))),
        }
    }
}

/// Represents a function parameter in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirParameter {
    name: String,
    param_type: HirType,
}

impl HirParameter {
    /// Create a new HIR parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_hir::{HirParameter, HirType};
    ///
    /// let param = HirParameter::new("x".to_string(), HirType::Int);
    /// assert_eq!(param.name(), "x");
    /// ```
    pub fn new(name: String, param_type: HirType) -> Self {
        Self { name, param_type }
    }

    /// Get the parameter name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the parameter type.
    pub fn param_type(&self) -> &HirType {
        &self.param_type
    }

    /// Convert from parser AST parameter to HIR parameter.
    pub fn from_ast_parameter(ast_param: &decy_parser::parser::Parameter) -> Self {
        Self {
            name: ast_param.name.clone(),
            param_type: HirType::from_ast_type(&ast_param.param_type),
        }
    }
}

/// Represents a function in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFunction {
    name: String,
    return_type: HirType,
    parameters: Vec<HirParameter>,
    body: Option<Vec<HirStatement>>,
}

impl HirFunction {
    /// Create a new HIR function.
    ///
    /// # Examples
    ///
    /// ```
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
    /// assert_eq!(func.name(), "add");
    /// assert_eq!(func.parameters().len(), 2);
    /// ```
    pub fn new(name: String, return_type: HirType, parameters: Vec<HirParameter>) -> Self {
        Self {
            name,
            return_type,
            parameters,
            body: None,
        }
    }

    /// Get the function name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the return type.
    pub fn return_type(&self) -> &HirType {
        &self.return_type
    }

    /// Get the parameters.
    pub fn parameters(&self) -> &[HirParameter] {
        &self.parameters
    }

    /// Convert from parser AST function to HIR function.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_hir::HirFunction;
    /// use decy_parser::parser::{Function, Type, Parameter};
    ///
    /// let ast_func = Function::new(
    ///     "test".to_string(),
    ///     Type::Void,
    ///     vec![],
    /// );
    ///
    /// let hir_func = HirFunction::from_ast_function(&ast_func);
    /// assert_eq!(hir_func.name(), "test");
    /// ```
    pub fn from_ast_function(ast_func: &decy_parser::parser::Function) -> Self {
        Self {
            name: ast_func.name.clone(),
            return_type: HirType::from_ast_type(&ast_func.return_type),
            parameters: ast_func
                .parameters
                .iter()
                .map(HirParameter::from_ast_parameter)
                .collect(),
            body: None,
        }
    }

    /// Create a new HIR function with a body.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_hir::{HirFunction, HirType, HirStatement, HirExpression};
    ///
    /// let func = HirFunction::new_with_body(
    ///     "test".to_string(),
    ///     HirType::Int,
    ///     vec![],
    ///     vec![
    ///         HirStatement::VariableDeclaration {
    ///             name: "x".to_string(),
    ///             var_type: HirType::Int,
    ///             initializer: Some(HirExpression::IntLiteral(5)),
    ///         },
    ///         HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
    ///     ],
    /// );
    ///
    /// assert_eq!(func.name(), "test");
    /// assert_eq!(func.body().len(), 2);
    /// ```
    pub fn new_with_body(
        name: String,
        return_type: HirType,
        parameters: Vec<HirParameter>,
        body: Vec<HirStatement>,
    ) -> Self {
        Self {
            name,
            return_type,
            parameters,
            body: Some(body),
        }
    }

    /// Get the function body.
    pub fn body(&self) -> &[HirStatement] {
        self.body.as_deref().unwrap_or(&[])
    }
}

/// Binary operators for expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Modulo (%)
    Modulo,
    /// Equality (==)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Greater than (>)
    GreaterThan,
    /// Less than or equal (<=)
    LessEqual,
    /// Greater than or equal (>=)
    GreaterEqual,
}

/// Represents an expression in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExpression {
    /// Integer literal
    IntLiteral(i32),
    /// Variable reference
    Variable(String),
    /// Binary operation (left op right)
    BinaryOp {
        /// The operator
        op: BinaryOperator,
        /// Left operand
        left: Box<HirExpression>,
        /// Right operand
        right: Box<HirExpression>,
    },
    /// Dereference operation (*ptr)
    Dereference(Box<HirExpression>),
    /// Address-of operation (&x)
    AddressOf(Box<HirExpression>),
    /// Function call (function_name(args...))
    FunctionCall {
        /// Function name
        function: String,
        /// Arguments
        arguments: Vec<HirExpression>,
    },
}

/// Represents a statement in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStatement {
    /// Variable declaration with optional initializer
    VariableDeclaration {
        /// Variable name
        name: String,
        /// Variable type
        var_type: HirType,
        /// Optional initializer expression
        initializer: Option<HirExpression>,
    },
    /// Return statement with optional value
    Return(Option<HirExpression>),
    /// If statement with condition, then-block, and optional else-block
    If {
        /// Condition expression
        condition: HirExpression,
        /// Then block (statements to execute if condition is true)
        then_block: Vec<HirStatement>,
        /// Else block (optional statements to execute if condition is false)
        else_block: Option<Vec<HirStatement>>,
    },
    /// While loop with condition and body
    While {
        /// Loop condition
        condition: HirExpression,
        /// Loop body (statements to execute while condition is true)
        body: Vec<HirStatement>,
    },
    /// Break statement (exit loop)
    Break,
    /// Continue statement (skip to next iteration)
    Continue,
}

#[cfg(test)]
#[path = "hir_tests.rs"]
mod hir_tests;

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;

#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
