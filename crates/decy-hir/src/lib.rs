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
        }
    }
}

#[cfg(test)]
#[path = "hir_tests.rs"]
mod hir_tests;

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;
