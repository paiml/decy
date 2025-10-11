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
    /// Boxed type (Rust `Box<T>`)
    Box(Box<HirType>),
    /// Vec type (Rust `Vec<T>`)
    Vec(Box<HirType>),
    /// Reference type (Rust `&T` or `&mut T`)
    Reference {
        /// Inner type
        inner: Box<HirType>,
        /// Whether the reference is mutable
        mutable: bool,
    },
    /// Struct type (by name)
    Struct(String),
    /// Enum type (by name)
    Enum(String),
    /// Array type with optional size (fixed-size or unsized)
    Array {
        /// Element type
        element_type: Box<HirType>,
        /// Optional size (None for unsized arrays like function parameters)
        size: Option<usize>,
    },
    /// Function pointer type (Rust `fn` type)
    FunctionPointer {
        /// Parameter types
        param_types: Vec<HirType>,
        /// Return type
        return_type: Box<HirType>,
    },
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

/// Represents a struct field in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStructField {
    name: String,
    field_type: HirType,
}

impl HirStructField {
    /// Create a new struct field.
    pub fn new(name: String, field_type: HirType) -> Self {
        Self { name, field_type }
    }

    /// Get the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the field type.
    pub fn field_type(&self) -> &HirType {
        &self.field_type
    }
}

/// Represents a struct definition in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStruct {
    name: String,
    fields: Vec<HirStructField>,
}

impl HirStruct {
    /// Create a new struct.
    pub fn new(name: String, fields: Vec<HirStructField>) -> Self {
        Self { name, fields }
    }

    /// Get the struct name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the struct fields.
    pub fn fields(&self) -> &[HirStructField] {
        &self.fields
    }
}

/// Represents an enum variant in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirEnumVariant {
    name: String,
    value: Option<i32>,
}

impl HirEnumVariant {
    /// Create a new enum variant.
    pub fn new(name: String, value: Option<i32>) -> Self {
        Self { name, value }
    }

    /// Get the variant name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the variant value.
    pub fn value(&self) -> Option<i32> {
        self.value
    }
}

/// Represents an enum definition in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirEnum {
    name: String,
    variants: Vec<HirEnumVariant>,
}

impl HirEnum {
    /// Create a new enum.
    pub fn new(name: String, variants: Vec<HirEnumVariant>) -> Self {
        Self { name, variants }
    }

    /// Get the enum name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the enum variants.
    pub fn variants(&self) -> &[HirEnumVariant] {
        &self.variants
    }
}

/// Represents a typedef (type alias) in HIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTypedef {
    name: String,
    underlying_type: HirType,
}

impl HirTypedef {
    /// Create a new typedef.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_hir::{HirTypedef, HirType};
    ///
    /// let typedef = HirTypedef::new("Integer".to_string(), HirType::Int);
    /// assert_eq!(typedef.name(), "Integer");
    /// assert_eq!(typedef.underlying_type(), &HirType::Int);
    /// ```
    pub fn new(name: String, underlying_type: HirType) -> Self {
        Self {
            name,
            underlying_type,
        }
    }

    /// Get the typedef name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the underlying type.
    pub fn underlying_type(&self) -> &HirType {
        &self.underlying_type
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
        let body = if ast_func.body.is_empty() {
            None
        } else {
            Some(
                ast_func
                    .body
                    .iter()
                    .map(HirStatement::from_ast_statement)
                    .collect(),
            )
        };

        Self {
            name: ast_func.name.clone(),
            return_type: HirType::from_ast_type(&ast_func.return_type),
            parameters: ast_func
                .parameters
                .iter()
                .map(HirParameter::from_ast_parameter)
                .collect(),
            body,
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
    /// String literal (C: "hello" → Rust: "hello")
    StringLiteral(String),
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
    /// Field access (obj.field)
    FieldAccess {
        /// Object expression
        object: Box<HirExpression>,
        /// Field name
        field: String,
    },
    /// Pointer field access (ptr->field)
    PointerFieldAccess {
        /// Pointer expression
        pointer: Box<HirExpression>,
        /// Field name
        field: String,
    },
    /// Array indexing (arr\[index\])
    ArrayIndex {
        /// Array expression
        array: Box<HirExpression>,
        /// Index expression
        index: Box<HirExpression>,
    },
}

/// Represents a single case in a switch statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchCase {
    /// Case value expression (None for default case)
    pub value: Option<HirExpression>,
    /// Statements to execute for this case
    pub body: Vec<HirStatement>,
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
    /// Assignment statement (target = value)
    Assignment {
        /// Target variable name
        target: String,
        /// Value expression to assign
        value: HirExpression,
    },
    /// For loop with optional init, condition, optional increment, and body
    For {
        /// Optional initialization statement (e.g., int i = 0)
        init: Option<Box<HirStatement>>,
        /// Loop condition expression
        condition: HirExpression,
        /// Optional increment statement (e.g., i++)
        increment: Option<Box<HirStatement>>,
        /// Loop body (statements to execute while condition is true)
        body: Vec<HirStatement>,
    },
    /// Switch statement with condition, cases, and optional default case
    Switch {
        /// Condition expression to match against
        condition: HirExpression,
        /// List of case statements
        cases: Vec<SwitchCase>,
        /// Optional default case body
        default_case: Option<Vec<HirStatement>>,
    },
}

impl HirStatement {
    /// Convert from parser AST statement to HIR statement.
    pub fn from_ast_statement(ast_stmt: &decy_parser::parser::Statement) -> Self {
        use decy_parser::parser::Statement;
        match ast_stmt {
            Statement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => HirStatement::VariableDeclaration {
                name: name.clone(),
                var_type: HirType::from_ast_type(var_type),
                initializer: initializer.as_ref().map(HirExpression::from_ast_expression),
            },
            Statement::Return(expr) => {
                HirStatement::Return(expr.as_ref().map(HirExpression::from_ast_expression))
            }
            Statement::Assignment { target, value } => HirStatement::Assignment {
                target: target.clone(),
                value: HirExpression::from_ast_expression(value),
            },
        }
    }
}

impl HirExpression {
    /// Convert from parser AST expression to HIR expression.
    pub fn from_ast_expression(ast_expr: &decy_parser::parser::Expression) -> Self {
        use decy_parser::parser::Expression;
        match ast_expr {
            Expression::IntLiteral(value) => HirExpression::IntLiteral(*value),
            Expression::Variable(name) => HirExpression::Variable(name.clone()),
            Expression::BinaryOp { op, left, right } => HirExpression::BinaryOp {
                op: convert_binary_operator(*op),
                left: Box::new(HirExpression::from_ast_expression(left)),
                right: Box::new(HirExpression::from_ast_expression(right)),
            },
            Expression::FunctionCall {
                function,
                arguments,
            } => HirExpression::FunctionCall {
                function: function.clone(),
                arguments: arguments
                    .iter()
                    .map(HirExpression::from_ast_expression)
                    .collect(),
            },
        }
    }
}

/// Convert parser BinaryOperator to HIR BinaryOperator
fn convert_binary_operator(op: decy_parser::parser::BinaryOperator) -> BinaryOperator {
    use decy_parser::parser::BinaryOperator as ParserOp;
    match op {
        ParserOp::Add => BinaryOperator::Add,
        ParserOp::Subtract => BinaryOperator::Subtract,
        ParserOp::Multiply => BinaryOperator::Multiply,
        ParserOp::Divide => BinaryOperator::Divide,
        ParserOp::Modulo => BinaryOperator::Modulo,
        ParserOp::Equal => BinaryOperator::Equal,
        ParserOp::NotEqual => BinaryOperator::NotEqual,
        ParserOp::LessThan => BinaryOperator::LessThan,
        ParserOp::GreaterThan => BinaryOperator::GreaterThan,
        ParserOp::LessEqual => BinaryOperator::LessEqual,
        ParserOp::GreaterEqual => BinaryOperator::GreaterEqual,
    }
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

#[cfg(test)]
#[path = "struct_tests.rs"]
mod struct_tests;

#[cfg(test)]
#[path = "array_indexing_tests.rs"]
mod array_indexing_tests;

#[cfg(test)]
#[path = "for_loop_tests.rs"]
mod for_loop_tests;

#[cfg(test)]
#[path = "typedef_tests.rs"]
mod typedef_tests;

#[cfg(test)]
#[path = "typedef_property_tests.rs"]
mod typedef_property_tests;

#[cfg(test)]
#[path = "function_pointer_tests.rs"]
mod function_pointer_tests;

#[cfg(test)]
#[path = "string_tests.rs"]
mod string_tests;

#[cfg(test)]
#[path = "string_property_tests.rs"]
mod string_property_tests;

#[cfg(test)]
#[path = "switch_tests.rs"]
mod switch_tests;
