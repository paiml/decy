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
use std::collections::HashMap;

/// Type context for tracking variable types and struct definitions during code generation.
/// Used to detect pointer arithmetic, null pointer assignments, and other type-specific operations.
#[derive(Debug, Clone)]
struct TypeContext {
    variables: HashMap<String, HirType>,
    structs: HashMap<String, Vec<(String, HirType)>>, // struct_name -> [(field_name, field_type)]
}

impl TypeContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    fn from_function(func: &HirFunction) -> Self {
        let mut ctx = Self::new();
        // Add parameters to context
        for param in func.parameters() {
            ctx.variables
                .insert(param.name().to_string(), param.param_type().clone());
        }
        ctx
    }

    fn add_variable(&mut self, name: String, var_type: HirType) {
        self.variables.insert(name, var_type);
    }

    fn add_struct(&mut self, struct_def: &decy_hir::HirStruct) {
        let fields: Vec<(String, HirType)> = struct_def
            .fields()
            .iter()
            .map(|f| (f.name().to_string(), f.field_type().clone()))
            .collect();
        self.structs.insert(struct_def.name().to_string(), fields);
    }

    fn get_type(&self, name: &str) -> Option<&HirType> {
        self.variables.get(name)
    }

    fn get_field_type(&self, object_expr: &HirExpression, field_name: &str) -> Option<HirType> {
        // Get the type of the object expression
        let object_type = match object_expr {
            HirExpression::Variable(var_name) => self.get_type(var_name)?,
            _ => return None,
        };

        // Extract the struct name from the type
        let struct_name = match object_type {
            HirType::Struct(name) => name,
            HirType::Pointer(inner) => {
                // If it's a pointer to a struct, dereference it
                if let HirType::Struct(name) = &**inner {
                    name
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        // Look up the field type in the struct definition
        let fields = self.structs.get(struct_name)?;
        fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, field_type)| field_type.clone())
    }

    fn is_pointer(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Pointer(_)))
    }

    fn is_option(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Option(_)))
    }

    /// Infer the type of an expression based on the context.
    /// Returns None if the type cannot be inferred.
    fn infer_expression_type(&self, expr: &HirExpression) -> Option<HirType> {
        match expr {
            HirExpression::Variable(name) => self.get_type(name).cloned(),
            HirExpression::Dereference(inner) => {
                // If inner is *mut T, then *inner is T
                if let Some(HirType::Pointer(pointee_type)) = self.infer_expression_type(inner) {
                    Some(*pointee_type)
                } else {
                    None
                }
            }
            HirExpression::ArrayIndex { array, index: _ } => {
                // If array is [T; N] or *mut T, then array[i] is T
                if let Some(array_type) = self.infer_expression_type(array) {
                    match array_type {
                        HirType::Array { element_type, .. } => Some(*element_type),
                        HirType::Pointer(element_type) => Some(*element_type),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

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
            HirType::Option(inner) => {
                format!("Option<{}>", Self::map_type(inner))
            }
            HirType::Reference { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", Self::map_type(inner))
                } else {
                    format!("&{}", Self::map_type(inner))
                }
            }
            HirType::Struct(name) => name.clone(),
            HirType::Enum(name) => name.clone(),
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!("[{}; {}]", Self::map_type(element_type), n)
                } else {
                    // Unsized array - use slice reference
                    format!("[{}]", Self::map_type(element_type))
                }
            }
            HirType::FunctionPointer {
                param_types,
                return_type,
            } => {
                // C: int (*func_ptr)(int, int); → Rust: fn(i32, i32) -> i32
                let params: Vec<String> = param_types.iter().map(Self::map_type).collect();
                let params_str = params.join(", ");

                // Skip return type annotation for void
                if matches!(**return_type, HirType::Void) {
                    format!("fn({})", params_str)
                } else {
                    format!("fn({}) -> {}", params_str, Self::map_type(return_type))
                }
            }
        }
    }

    /// Map C type name from sizeof to Rust type string.
    ///
    /// Handles type names as strings from sizeof expressions.
    /// Examples: "int" → "i32", "struct Data" → "Data"
    fn map_sizeof_type(&self, c_type_name: &str) -> String {
        let trimmed = c_type_name.trim();

        // Handle basic C types
        match trimmed {
            "int" => "i32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "char" => "u8".to_string(),
            "void" => "()".to_string(),
            _ => {
                // Handle "struct TypeName" → "TypeName"
                if let Some(struct_name) = trimmed.strip_prefix("struct ") {
                    struct_name.trim().to_string()
                } else {
                    // Keep custom type names as-is
                    trimmed.to_string()
                }
            }
        }
    }

    /// Generate code for an expression.
    #[allow(clippy::only_used_in_recursion)]
    pub fn generate_expression(&self, expr: &HirExpression) -> String {
        self.generate_expression_with_context(expr, &TypeContext::new())
    }

    /// Generate code for an expression with type context for pointer arithmetic.
    #[allow(clippy::only_used_in_recursion)]
    fn generate_expression_with_context(&self, expr: &HirExpression, ctx: &TypeContext) -> String {
        self.generate_expression_with_target_type(expr, ctx, None)
    }

    /// Generate code for an expression with optional target type hint for null pointer detection.
    /// If target_type is Some(HirType::Pointer(_)) and expr is IntLiteral(0), generates std::ptr::null_mut().
    #[allow(clippy::only_used_in_recursion)]
    fn generate_expression_with_target_type(
        &self,
        expr: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match expr {
            HirExpression::IntLiteral(val) => {
                // Check if assigning 0 to a pointer type
                if *val == 0 {
                    if let Some(HirType::Pointer(_)) = target_type {
                        return "std::ptr::null_mut()".to_string();
                    }
                }
                val.to_string()
            }
            HirExpression::StringLiteral(s) => format!("\"{}\"", s),
            HirExpression::Variable(name) => name.clone(),
            HirExpression::BinaryOp { op, left, right } => {
                // Check for Option comparison with NULL → is_none() / is_some()
                // p == NULL → p.is_none(), p != NULL → p.is_some()
                if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                    // Check if left is an Option and right is NULL
                    if let HirExpression::Variable(var_name) = &**left {
                        if ctx.is_option(var_name) && matches!(**right, HirExpression::NullLiteral)
                        {
                            return match op {
                                BinaryOperator::Equal => format!("{}.is_none()", var_name),
                                BinaryOperator::NotEqual => format!("{}.is_some()", var_name),
                                _ => unreachable!(),
                            };
                        }
                    }
                    // Check if right is an Option and left is NULL (NULL == p or NULL != p)
                    if let HirExpression::Variable(var_name) = &**right {
                        if ctx.is_option(var_name) && matches!(**left, HirExpression::NullLiteral) {
                            return match op {
                                BinaryOperator::Equal => format!("{}.is_none()", var_name),
                                BinaryOperator::NotEqual => format!("{}.is_some()", var_name),
                                _ => unreachable!(),
                            };
                        }
                    }

                    // Check for pointer comparison with 0 (null pointer comparison)
                    // ptr == 0 or ptr != 0 should become ptr == std::ptr::null_mut() or ptr != std::ptr::null_mut()
                    // Check if left is a pointer and right is 0
                    if let HirExpression::Variable(var_name) = &**left {
                        if ctx.is_pointer(var_name) {
                            if let HirExpression::IntLiteral(0) = **right {
                                let op_str = Self::binary_operator_to_string(op);
                                return format!("{} {} std::ptr::null_mut()", var_name, op_str);
                            }
                        }
                    }
                    // Check if right is a pointer and left is 0 (0 == ptr or 0 != ptr)
                    if let HirExpression::Variable(var_name) = &**right {
                        if ctx.is_pointer(var_name) {
                            if let HirExpression::IntLiteral(0) = **left {
                                let op_str = Self::binary_operator_to_string(op);
                                return format!("std::ptr::null_mut() {} {}", op_str, var_name);
                            }
                        }
                    }
                }

                let left_code = self.generate_expression_with_context(left, ctx);
                let right_code = self.generate_expression_with_context(right, ctx);
                let op_str = Self::binary_operator_to_string(op);

                // Add parentheses for nested binary operations
                let left_str = if matches!(**left, HirExpression::BinaryOp { .. }) {
                    format!("({})", left_code)
                } else {
                    left_code.clone()
                };

                let right_str = if matches!(**right, HirExpression::BinaryOp { .. }) {
                    format!("({})", right_code)
                } else {
                    right_code.clone()
                };

                // DECY-041: Detect pointer arithmetic using type context
                if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                    if let HirExpression::Variable(var_name) = &**left {
                        if ctx.is_pointer(var_name) {
                            // This is pointer arithmetic - generate unsafe pointer method calls
                            return match op {
                                BinaryOperator::Add => {
                                    format!(
                                        "unsafe {{ {}.wrapping_add({} as usize) }}",
                                        left_str, right_str
                                    )
                                }
                                BinaryOperator::Subtract => {
                                    // Check if right is also a pointer (ptr - ptr) or integer (ptr - offset)
                                    if let HirExpression::Variable(right_var) = &**right {
                                        if ctx.is_pointer(right_var) {
                                            // ptr - ptr: calculate difference (returns isize, cast to i32 for C compatibility)
                                            format!(
                                                "unsafe {{ {}.offset_from({}) as i32 }}",
                                                left_str, right_str
                                            )
                                        } else {
                                            // ptr - integer offset
                                            format!(
                                                "unsafe {{ {}.wrapping_sub({} as usize) }}",
                                                left_str, right_str
                                            )
                                        }
                                    } else {
                                        // ptr - integer offset (literal or expression)
                                        format!(
                                            "unsafe {{ {}.wrapping_sub({} as usize) }}",
                                            left_str, right_str
                                        )
                                    }
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                }

                format!("{} {} {}", left_str, op_str, right_str)
            }
            HirExpression::Dereference(inner) => {
                let inner_code = self.generate_expression_with_context(inner, ctx);

                // DECY-041: Check if dereferencing a raw pointer - if so, wrap in unsafe
                if let HirExpression::Variable(var_name) = &**inner {
                    if ctx.is_pointer(var_name) {
                        return format!("unsafe {{ *{} }}", inner_code);
                    }
                }

                format!("*{}", inner_code)
            }
            HirExpression::AddressOf(inner) => {
                let inner_code = self.generate_expression_with_context(inner, ctx);
                // Add parentheses for non-trivial expressions
                if matches!(**inner, HirExpression::Dereference(_)) {
                    format!("&({})", inner_code)
                } else {
                    format!("&{}", inner_code)
                }
            }
            HirExpression::UnaryOp { op, operand } => {
                let op_str = Self::unary_operator_to_string(op);
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("{}{}", op_str, operand_code)
            }
            HirExpression::FunctionCall {
                function,
                arguments,
            } => {
                let args: Vec<String> = arguments
                    .iter()
                    .map(|arg| self.generate_expression_with_context(arg, ctx))
                    .collect();
                format!("{}({})", function, args.join(", "))
            }
            HirExpression::FieldAccess { object, field } => {
                format!(
                    "{}.{}",
                    self.generate_expression_with_context(object, ctx),
                    field
                )
            }
            HirExpression::PointerFieldAccess { pointer, field } => {
                // In Rust, ptr->field becomes (*ptr).field
                // However, if the pointer is already a field access (ptr->field1->field2),
                // we should generate (*ptr).field1.field2 not (*(*ptr).field1).field2
                match &**pointer {
                    // If the pointer is itself a field access expression, we can chain with .
                    HirExpression::PointerFieldAccess { .. }
                    | HirExpression::FieldAccess { .. } => {
                        format!(
                            "{}.{}",
                            self.generate_expression_with_context(pointer, ctx),
                            field
                        )
                    }
                    // For other expressions (variables, array index, etc), we need explicit deref
                    _ => {
                        format!(
                            "(*{}).{}",
                            self.generate_expression_with_context(pointer, ctx),
                            field
                        )
                    }
                }
            }
            HirExpression::ArrayIndex { array, index } => {
                let array_code = self.generate_expression_with_context(array, ctx);
                let index_code = self.generate_expression_with_context(index, ctx);

                // DECY-041: Check if array is a raw pointer - if so, use unsafe pointer arithmetic
                if let HirExpression::Variable(var_name) = &**array {
                    if ctx.is_pointer(var_name) {
                        // Raw pointer indexing: arr[i] becomes unsafe { *arr.add(i as usize) }
                        return format!(
                            "unsafe {{ *{}.add({} as usize) }}",
                            array_code, index_code
                        );
                    }
                }

                // Regular array/slice indexing
                format!("{}[{}]", array_code, index_code)
            }
            HirExpression::Sizeof { type_name } => {
                // sizeof(int) → std::mem::size_of::<i32>() as i32
                // sizeof(struct Data) → std::mem::size_of::<Data>() as i32
                // Note: size_of returns usize, but C's sizeof returns int (typically i32)
                let rust_type = self.map_sizeof_type(type_name);
                format!("std::mem::size_of::<{}>() as i32", rust_type)
            }
            HirExpression::NullLiteral => {
                // NULL → None
                "None".to_string()
            }
            HirExpression::IsNotNull(inner) => {
                // p != NULL → if let Some(p) = p
                // This is a helper expression for generating Option checks
                // In actual codegen, we transform if (p) to if let Some(_) = p
                let inner_code = self.generate_expression_with_context(inner, ctx);
                format!("if let Some(_) = {}", inner_code)
            }
            HirExpression::Calloc {
                count,
                element_type,
            } => {
                // calloc(n, sizeof(T)) → vec![0T; n]
                // Generate zero-initialized vec![default; count]
                let count_code = self.generate_expression_with_context(count, ctx);

                // Get default value with type suffix for clarity
                let default_value = match element_type.as_ref() {
                    HirType::Int => "0i32",
                    HirType::Float => "0.0f32",
                    HirType::Double => "0.0f64",
                    HirType::Char => "0u8",
                    _ => &Self::default_value_for_type(element_type),
                };

                format!("vec![{}; {}]", default_value, count_code)
            }
            HirExpression::Malloc { size } => {
                // malloc(size) should have been transformed to Box or Vec by analyzer
                // If we're generating this directly, treat it as Box::new(default)
                // Note: The proper transformation should happen at HIR level via PatternDetector

                // Try to detect if this is an array allocation (n * sizeof(T))
                // If so, generate Vec::with_capacity
                if let HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left,
                    ..
                } = size.as_ref()
                {
                    // This looks like n * sizeof(T) → Vec::with_capacity(n)
                    let capacity_code = self.generate_expression_with_context(left, ctx);
                    format!("Vec::with_capacity({})", capacity_code)
                } else {
                    // Single allocation → Box::new(default)
                    "Box::new(0i32)".to_string()
                }
            }
            HirExpression::Realloc { pointer, new_size } => {
                // realloc(ptr, new_size) transformation depends on context:
                // 1. realloc(NULL, size) → treat as malloc (Vec allocation)
                // 2. realloc(ptr, 0) → treat as free (RAII comment or clear)
                // 3. realloc(ptr, new_size) → vec.resize(new_count, default)
                //
                // Since we're generating an expression here, we'll return a placeholder
                // The actual transformation should happen in Assignment statement handling
                // For now, just generate a comment indicating this needs special handling

                // Check if pointer is NULL → malloc equivalent
                if matches!(**pointer, HirExpression::NullLiteral) {
                    // realloc(NULL, size) → vec![default; count]
                    if let HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left,
                        ..
                    } = new_size.as_ref()
                    {
                        let count_code = self.generate_expression_with_context(left, ctx);
                        format!("vec![0i32; {}]", count_code)
                    } else {
                        "Vec::new()".to_string()
                    }
                } else {
                    // realloc(ptr, size) - this should be handled at statement level
                    // For expression context, return the pointer unchanged as a placeholder
                    self.generate_expression_with_context(pointer, ctx)
                }
            }
        }
    }

    /// Convert unary operator to string.
    fn unary_operator_to_string(op: &decy_hir::UnaryOperator) -> &'static str {
        use decy_hir::UnaryOperator;
        match op {
            UnaryOperator::Minus => "-",
            UnaryOperator::LogicalNot => "!",
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
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::LogicalOr => "||",
        }
    }

    /// Get default value for a type (for uninitialized variables).
    fn default_value_for_type(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "0i32".to_string(),
            HirType::Float => "0.0f32".to_string(),
            HirType::Double => "0.0f64".to_string(),
            HirType::Char => "0u8".to_string(),
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
            HirType::Option(_) => {
                // Option types default to None
                "None".to_string()
            }
            HirType::Reference { .. } => {
                // References cannot have default values - they must always be initialized
                // This should never be reached in valid code
                panic!("References must be initialized and cannot have default values")
            }
            HirType::Struct(name) => {
                format!("{}::default()", name)
            }
            HirType::Enum(name) => {
                format!("{}::default()", name)
            }
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!("[{}; {}]", Self::default_value_for_type(element_type), n)
                } else {
                    // Unsized arrays cannot have default values - this should be initialized from parameter
                    panic!("Unsized arrays must be initialized and cannot have default values")
                }
            }
            HirType::FunctionPointer { .. } => {
                // Function pointers cannot have meaningful default values
                // They must be initialized with an actual function
                panic!("Function pointers must be initialized and cannot have default values")
            }
        }
    }

    /// Generate code for a statement.
    pub fn generate_statement(&self, stmt: &HirStatement) -> String {
        self.generate_statement_for_function(stmt, None)
    }

    /// Generate code for a statement, with optional function context.
    ///
    /// When function_name is "main", special handling applies (DECY-AUDIT-001):
    /// - return N; becomes std::process::exit(N);
    fn generate_statement_for_function(
        &self,
        stmt: &HirStatement,
        function_name: Option<&str>,
    ) -> String {
        self.generate_statement_with_context(stmt, function_name, &mut TypeContext::new(), None)
    }

    /// Generate code for a statement with type context for pointer arithmetic and return type for null pointer detection.
    fn generate_statement_with_context(
        &self,
        stmt: &HirStatement,
        function_name: Option<&str>,
        ctx: &mut TypeContext,
        return_type: Option<&HirType>,
    ) -> String {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                // Add variable to type context for pointer arithmetic detection
                ctx.add_variable(name.clone(), var_type.clone());

                let mut code = format!("let mut {}: {}", name, Self::map_type(var_type));
                if let Some(init_expr) = initializer {
                    // Special handling for Malloc expressions - use var_type to generate correct Box::new
                    if matches!(init_expr, HirExpression::Malloc { .. }) {
                        // Malloc → Box::new or Vec (depending on var_type)
                        match var_type {
                            HirType::Box(inner) => {
                                code.push_str(&format!(
                                    " = Box::new({});",
                                    Self::default_value_for_type(inner)
                                ));
                            }
                            HirType::Vec(_) => {
                                // Extract capacity from malloc size expression
                                if let HirExpression::Malloc { size } = init_expr {
                                    if let HirExpression::BinaryOp {
                                        op: decy_hir::BinaryOperator::Multiply,
                                        left,
                                        ..
                                    } = size.as_ref()
                                    {
                                        let capacity_code =
                                            self.generate_expression_with_context(left, ctx);
                                        code.push_str(&format!(
                                            " = Vec::with_capacity({});",
                                            capacity_code
                                        ));
                                    } else {
                                        code.push_str(" = Vec::new();");
                                    }
                                } else {
                                    code.push_str(" = Vec::new();");
                                }
                            }
                            _ => {
                                // Default to Box::new(0i32) for other types
                                code.push_str(" = Box::new(0i32);");
                            }
                        }
                    } else {
                        // Pass var_type as target type hint for null pointer detection
                        code.push_str(&format!(
                            " = {};",
                            self.generate_expression_with_target_type(
                                init_expr,
                                ctx,
                                Some(var_type)
                            )
                        ));
                    }
                } else {
                    // Provide default value for uninitialized variables
                    code.push_str(&format!(" = {};", Self::default_value_for_type(var_type)));
                }
                code
            }
            HirStatement::Return(expr_opt) => {
                // Special handling for main function (DECY-AUDIT-001)
                // return N; in main becomes std::process::exit(N);
                if function_name == Some("main") {
                    if let Some(expr) = expr_opt {
                        format!(
                            "std::process::exit({});",
                            self.generate_expression_with_context(expr, ctx)
                        )
                    } else {
                        "std::process::exit(0);".to_string()
                    }
                } else if let Some(expr) = expr_opt {
                    // Pass return type as target type hint for null pointer detection
                    format!(
                        "return {};",
                        self.generate_expression_with_target_type(expr, ctx, return_type)
                    )
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
                code.push_str(&format!(
                    "if {} {{\n",
                    self.generate_expression_with_context(condition, ctx)
                ));

                // Generate then block
                for stmt in then_block {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement_with_context(
                        stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }

                // Generate else block if present
                if let Some(else_stmts) = else_block {
                    code.push_str("} else {\n");
                    for stmt in else_stmts {
                        code.push_str("    ");
                        code.push_str(&self.generate_statement_with_context(
                            stmt,
                            function_name,
                            ctx,
                            return_type,
                        ));
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
                    self.generate_expression_with_context(condition, ctx)
                ));

                // Generate loop body
                for stmt in body {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement_with_context(
                        stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }

                code.push('}');
                code
            }
            HirStatement::Break => "break;".to_string(),
            HirStatement::Continue => "continue;".to_string(),
            HirStatement::Assignment { target, value } => {
                // Special handling for realloc() → Vec::resize/truncate/clear
                if let HirExpression::Realloc { pointer, new_size } = value {
                    // target is a String (variable name) in Assignment statements
                    let target_var = target.clone();

                    // Check if target is a Vec type to get element type
                    let element_type = if let Some(HirType::Vec(inner)) = ctx.get_type(&target_var)
                    {
                        inner.as_ref().clone()
                    } else {
                        // Fallback: assume i32
                        HirType::Int
                    };

                    // Check special cases:
                    // 1. realloc(ptr, 0) → clear or RAII comment
                    if let HirExpression::IntLiteral(0) = **new_size {
                        return format!("{}.clear(); // Free equivalent: clear vector", target_var);
                    }

                    // 2. realloc(NULL, size) → should not appear in assignment (would be in initializer)
                    //    but handle it gracefully if it does
                    if matches!(**pointer, HirExpression::NullLiteral) {
                        // This is essentially malloc - but in assignment context, we'll treat it as resize from 0
                        if let HirExpression::BinaryOp {
                            op: decy_hir::BinaryOperator::Multiply,
                            left,
                            ..
                        } = new_size.as_ref()
                        {
                            let count_code = self.generate_expression_with_context(left, ctx);
                            let default_value = Self::default_value_for_type(&element_type);
                            return format!(
                                "{}.resize({}, {})",
                                target_var, count_code, default_value
                            );
                        }
                    }

                    // 3. realloc(ptr, new_size) → vec.resize(new_count, default)
                    // Extract count from new_size (typically n * sizeof(T))
                    if let HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left,
                        ..
                    } = new_size.as_ref()
                    {
                        let count_code = self.generate_expression_with_context(left, ctx);
                        let default_value = Self::default_value_for_type(&element_type);
                        format!("{}.resize({}, {});", target_var, count_code, default_value)
                    } else {
                        // Fallback: if new_size is not n * sizeof(T), generate direct resize
                        // This handles edge cases where size isn't n * sizeof(T)
                        let size_expr = self.generate_expression_with_context(new_size, ctx);
                        let default_value = Self::default_value_for_type(&element_type);
                        format!(
                            "{}.resize({} as usize, {});",
                            target_var, size_expr, default_value
                        )
                    }
                } else {
                    // Regular assignment (not realloc)
                    let target_type = ctx.get_type(target);
                    format!(
                        "{} = {};",
                        target,
                        self.generate_expression_with_target_type(value, ctx, target_type)
                    )
                }
            }
            HirStatement::For {
                init,
                condition,
                increment,
                body,
            } => {
                let mut code = String::new();

                // Generate init statement before loop (if present)
                if let Some(init_stmt) = init {
                    code.push_str(&self.generate_statement_with_context(
                        init_stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }

                // Generate while loop with condition
                code.push_str(&format!(
                    "while {} {{\n",
                    self.generate_expression_with_context(condition, ctx)
                ));

                // Generate loop body
                for stmt in body {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement_with_context(
                        stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }

                // Generate increment at end of body (if present)
                if let Some(inc_stmt) = increment {
                    code.push_str("    ");
                    code.push_str(&self.generate_statement_with_context(
                        inc_stmt,
                        function_name,
                        ctx,
                        return_type,
                    ));
                    code.push('\n');
                }

                code.push('}');
                code
            }
            HirStatement::Switch {
                condition,
                cases,
                default_case,
            } => {
                let mut code = String::new();

                // Generate match expression
                code.push_str(&format!(
                    "match {} {{\n",
                    self.generate_expression_with_context(condition, ctx)
                ));

                // Generate each case
                for case in cases {
                    if let Some(value_expr) = &case.value {
                        // Generate case pattern
                        code.push_str(&format!(
                            "    {} => {{\n",
                            self.generate_expression_with_context(value_expr, ctx)
                        ));

                        // Generate case body (filter out Break statements)
                        for stmt in &case.body {
                            if !matches!(stmt, HirStatement::Break) {
                                code.push_str("        ");
                                code.push_str(&self.generate_statement_with_context(
                                    stmt,
                                    function_name,
                                    ctx,
                                    return_type,
                                ));
                                code.push('\n');
                            }
                        }

                        code.push_str("    },\n");
                    }
                }

                // Generate default case (or empty default if not present)
                code.push_str("    _ => {\n");
                if let Some(default_stmts) = default_case {
                    for stmt in default_stmts {
                        if !matches!(stmt, HirStatement::Break) {
                            code.push_str("        ");
                            code.push_str(&self.generate_statement_with_context(
                                stmt,
                                function_name,
                                ctx,
                                return_type,
                            ));
                            code.push('\n');
                        }
                    }
                }
                code.push_str("    },\n");

                code.push('}');
                code
            }
            HirStatement::DerefAssignment { target, value } => {
                // Infer the type of *target for null pointer detection
                let target_type = ctx
                    .infer_expression_type(&HirExpression::Dereference(Box::new(target.clone())));
                format!(
                    "*{} = {};",
                    self.generate_expression_with_context(target, ctx),
                    self.generate_expression_with_target_type(value, ctx, target_type.as_ref())
                )
            }
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => {
                // Infer the type of array[index] for null pointer detection
                let target_expr = HirExpression::ArrayIndex {
                    array: array.clone(),
                    index: index.clone(),
                };
                let target_type = ctx.infer_expression_type(&target_expr);
                format!(
                    "{}[{}] = {};",
                    self.generate_expression_with_context(array, ctx),
                    self.generate_expression_with_context(index, ctx),
                    self.generate_expression_with_target_type(value, ctx, target_type.as_ref())
                )
            }
            HirStatement::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Look up field type for null pointer detection
                let field_type = ctx.get_field_type(object, field);
                // Generate obj.field = value (works for both ptr->field and obj.field in Rust)
                format!(
                    "{}.{} = {};",
                    self.generate_expression_with_context(object, ctx),
                    field,
                    self.generate_expression_with_target_type(value, ctx, field_type.as_ref())
                )
            }
            HirStatement::Free { pointer } => {
                // free(ptr) → automatic drop via RAII
                // Generate a comment explaining that the memory will be deallocated automatically
                // when the variable goes out of scope
                let pointer_name = match pointer {
                    HirExpression::Variable(name) => name.clone(),
                    _ => self.generate_expression_with_context(pointer, ctx),
                };
                format!(
                    "// Memory for '{}' deallocated automatically by RAII",
                    pointer_name
                )
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

        // Special handling for main function (DECY-AUDIT-001)
        // C's int main() must become Rust's fn main() (no return type)
        // Rust's entry point returns () and uses std::process::exit(N) for exit codes
        if func.name() == "main" && matches!(func.return_type(), HirType::Int) {
            // Skip return type for main - it must be fn main()
            return sig;
        }

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
                // DECY-041: Add mut for all parameters to match C semantics
                // In C, parameters are mutable by default (can be reassigned)
                // DECY-FUTURE: More sophisticated analysis to only add mut when needed
                format!(
                    "mut {}: {}",
                    p.name,
                    self.annotated_type_to_string(&p.param_type)
                )
            })
            .collect();
        result.push_str(&params.join(", "));
        result.push(')');

        // Special handling for main function (DECY-AUDIT-001)
        // C's int main() must become Rust's fn main() (no return type)
        // Rust's entry point returns () and uses std::process::exit(N) for exit codes
        let return_type_str = self.annotated_type_to_string(&sig.return_type);
        if sig.name == "main" && return_type_str == "i32" {
            // Skip return type for main - it must be fn main()
            return result;
        }

        // Add return type if not void
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
            HirType::Option(_) => "    return None;".to_string(),
            HirType::Reference { .. } => {
                // References in return position need concrete values from parameters
                // This should be handled by lifetime-annotated code generation
                // using generate_function_with_lifetimes() instead
                String::new()
            }
            HirType::Struct(name) => {
                format!("    return {}::default();", name)
            }
            HirType::Enum(name) => {
                format!("    return {}::default();", name)
            }
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!(
                        "    return [{}; {}];",
                        Self::default_value_for_type(element_type),
                        n
                    )
                } else {
                    // Unsized arrays in return position don't make sense
                    String::new()
                }
            }
            HirType::FunctionPointer { .. } => {
                // Function pointers in return position need concrete function values
                // This should be handled by the function body
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

        // Initialize type context for tracking variable types across statements
        let mut ctx = TypeContext::from_function(func);

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements with persistent context
            for stmt in func.body() {
                code.push_str("    ");
                code.push_str(&self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(func.return_type()),
                ));
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
        self.generate_function_with_lifetimes_and_structs(func, sig, &[])
    }

    /// Generate a complete function from HIR with lifetime annotations and struct definitions.
    ///
    /// Takes the HIR function, its annotated signature, and struct definitions to generate
    /// Rust code with proper lifetime annotations and field type awareness for null pointer detection.
    pub fn generate_function_with_lifetimes_and_structs(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
        structs: &[decy_hir::HirStruct],
    ) -> String {
        let mut code = String::new();

        // Generate signature with lifetimes
        code.push_str(&self.generate_annotated_signature(sig));
        code.push_str(" {\n");

        // DECY-041: Initialize type context with function parameters for pointer arithmetic
        let mut ctx = TypeContext::from_function(func);

        // Add struct definitions to context for field type lookup
        for struct_def in structs {
            ctx.add_struct(struct_def);
        }

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements with type context and return type
            for stmt in func.body() {
                code.push_str("    ");
                code.push_str(&self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(func.return_type()),
                ));
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
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
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
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
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
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a struct definition from HIR.
    ///
    /// Generates Rust struct code with automatic derives for Debug, Clone, PartialEq, Eq.
    /// Handles lifetimes automatically for structs with reference fields.
    pub fn generate_struct(&self, hir_struct: &decy_hir::HirStruct) -> String {
        let mut code = String::new();

        // Check if struct needs lifetimes (has Reference fields)
        let needs_lifetimes = hir_struct
            .fields()
            .iter()
            .any(|f| matches!(f.field_type(), HirType::Reference { .. }));

        // Add derive attribute
        code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");

        // Add struct declaration with or without lifetime
        if needs_lifetimes {
            code.push_str(&format!("pub struct {}<'a> {{\n", hir_struct.name()));
        } else {
            code.push_str(&format!("pub struct {} {{\n", hir_struct.name()));
        }

        // Add fields
        for field in hir_struct.fields() {
            code.push_str(&format!(
                "    pub {}: {},\n",
                field.name(),
                Self::map_type(field.field_type())
            ));
        }

        code.push('}');
        code
    }

    /// Generate an enum definition from HIR.
    ///
    /// Generates Rust enum code with automatic derives for Debug, Clone, Copy, PartialEq, Eq.
    /// Supports both simple enums and enums with explicit integer values.
    pub fn generate_enum(&self, hir_enum: &decy_hir::HirEnum) -> String {
        let mut code = String::new();

        // Add derive attribute (includes Copy since C enums are copyable)
        code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");

        // Add enum declaration
        code.push_str(&format!("pub enum {} {{\n", hir_enum.name()));

        // Add variants
        for variant in hir_enum.variants() {
            if let Some(value) = variant.value() {
                code.push_str(&format!("    {} = {},\n", variant.name(), value));
            } else {
                code.push_str(&format!("    {},\n", variant.name()));
            }
        }

        code.push('}');
        code
    }

    /// Generate a typedef (type alias) from HIR.
    ///
    /// Generates Rust type alias code using the `type` keyword.
    /// Handles redundant typedefs (where name matches underlying struct/enum name) as comments.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirTypedef, HirType};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Simple typedef: typedef int Integer;
    /// let typedef = HirTypedef::new("Integer".to_string(), HirType::Int);
    /// let code = codegen.generate_typedef(&typedef);
    /// assert!(code.contains("type Integer = i32"));
    ///
    /// // Pointer typedef: typedef int* IntPtr;
    /// let typedef = HirTypedef::new("IntPtr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    /// let code = codegen.generate_typedef(&typedef);
    /// assert!(code.contains("type IntPtr = *mut i32"));
    /// ```
    pub fn generate_typedef(&self, typedef: &decy_hir::HirTypedef) -> String {
        // Check for redundant typedef (struct/enum name matching typedef name)
        match typedef.underlying_type() {
            HirType::Struct(name) | HirType::Enum(name) if name == typedef.name() => {
                // In Rust, struct/enum names are already types, so this is redundant
                // Generate as a comment for documentation purposes
                format!("// type {} = {}; (redundant in Rust)", typedef.name(), name)
            }
            _ => {
                // Regular type alias
                format!(
                    "type {} = {};",
                    typedef.name(),
                    Self::map_type(typedef.underlying_type())
                )
            }
        }
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

#[cfg(test)]
#[path = "struct_codegen_tests.rs"]
mod struct_codegen_tests;

#[cfg(test)]
#[path = "for_loop_codegen_tests.rs"]
mod for_loop_codegen_tests;

#[cfg(test)]
#[path = "typedef_codegen_tests.rs"]
mod typedef_codegen_tests;

#[cfg(test)]
#[path = "typedef_property_tests.rs"]
mod typedef_property_tests;

#[cfg(test)]
#[path = "function_pointer_codegen_tests.rs"]
mod function_pointer_codegen_tests;

#[cfg(test)]
#[path = "string_codegen_tests.rs"]
mod string_codegen_tests;

#[cfg(test)]
#[path = "string_property_tests.rs"]
mod string_property_tests;

#[cfg(test)]
#[path = "switch_codegen_tests.rs"]
mod switch_codegen_tests;

#[cfg(test)]
#[path = "switch_property_tests.rs"]
mod switch_property_tests;
