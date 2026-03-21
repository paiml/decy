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
pub mod concurrency_transform;
pub mod enum_gen;
pub mod pattern_gen;
pub mod test_generator;

use decy_hir::{HirExpression, HirFunction, HirType};
use std::collections::HashMap;

/// Type context for tracking variable types and struct definitions during code generation.
/// Used to detect pointer arithmetic, null pointer assignments, and other type-specific operations.
#[derive(Debug, Clone)]
struct TypeContext {
    variables: HashMap<String, HirType>,
    structs: HashMap<String, Vec<(String, HirType)>>, // struct_name -> [(field_name, field_type)]
    // DECY-117: Track function signatures for call site reference mutability
    functions: HashMap<String, Vec<HirType>>, // func_name -> [param_types]
    // DECY-116: Track which argument indices to skip at call sites (removed length params)
    // func_name -> [(array_arg_index, len_arg_index_to_skip)]
    slice_func_args: HashMap<String, Vec<(usize, usize)>>,
    // DECY-134: Track string iteration params that use index-based access
    // Maps param_name -> index_var_name (e.g., "dest" -> "dest_idx")
    string_iter_params: HashMap<String, String>,
    // DECY-134b: Track which functions have string iteration params (for call site transformation)
    // Maps func_name -> list of (param_index, is_mutable) for string iter params
    string_iter_funcs: HashMap<String, Vec<(usize, bool)>>,
    // DECY-220: Track global variables (static mut) that need unsafe access
    globals: std::collections::HashSet<String>,
    // DECY-245: Track locals renamed to avoid shadowing statics (original_name -> renamed_name)
    renamed_locals: HashMap<String, String>,
}

impl TypeContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            slice_func_args: HashMap::new(),
            string_iter_params: HashMap::new(),
            string_iter_funcs: HashMap::new(),
            globals: std::collections::HashSet::new(),
            renamed_locals: HashMap::new(),
        }
    }

    /// DECY-245: Register a renamed local variable (for shadowing statics)
    fn add_renamed_local(&mut self, original: String, renamed: String) {
        self.renamed_locals.insert(original, renamed);
    }

    /// DECY-245: Get the renamed name for a local variable (if renamed)
    fn get_renamed_local(&self, name: &str) -> Option<&String> {
        self.renamed_locals.get(name)
    }

    /// DECY-220: Register a global variable (static mut) for unsafe tracking
    fn add_global(&mut self, name: String) {
        self.globals.insert(name);
    }

    /// DECY-220: Check if a variable is a global (static mut) requiring unsafe
    fn is_global(&self, name: &str) -> bool {
        self.globals.contains(name)
    }

    /// DECY-134b: Register a function's string iteration params for call site transformation
    fn add_string_iter_func(&mut self, func_name: String, params: Vec<(usize, bool)>) {
        self.string_iter_funcs.insert(func_name, params);
    }

    /// DECY-134b: Get string iteration param info for a function
    fn get_string_iter_func(&self, func_name: &str) -> Option<&Vec<(usize, bool)>> {
        self.string_iter_funcs.get(func_name)
    }

    /// DECY-134: Register a string iteration param with its index variable name
    fn add_string_iter_param(&mut self, param_name: String, index_var: String) {
        self.string_iter_params.insert(param_name, index_var);
    }

    /// DECY-134: Check if a variable is a string iteration param.
    #[cfg(test)]
    fn is_string_iter_param(&self, name: &str) -> bool {
        self.string_iter_params.contains_key(name)
    }

    /// DECY-134: Get the index variable name for a string iteration param
    fn get_string_iter_index(&self, name: &str) -> Option<&String> {
        self.string_iter_params.get(name)
    }

    /// DECY-117: Register a function signature for call site reference mutability
    fn add_function(&mut self, name: String, param_types: Vec<HirType>) {
        self.functions.insert(name, param_types);
    }

    /// DECY-116: Register which args to skip at call sites for slice functions
    fn add_slice_func_args(&mut self, name: String, arg_mappings: Vec<(usize, usize)>) {
        self.slice_func_args.insert(name, arg_mappings);
    }

    /// DECY-116: Get the arg indices to skip for a function (length params removed)
    fn get_slice_func_len_indices(&self, func_name: &str) -> Option<&Vec<(usize, usize)>> {
        self.slice_func_args.get(func_name)
    }

    /// DECY-117: Get the expected parameter type for a function call
    fn get_function_param_type(&self, func_name: &str, param_index: usize) -> Option<&HirType> {
        self.functions.get(func_name).and_then(|params| params.get(param_index))
    }

    fn from_function(func: &HirFunction) -> Self {
        let mut ctx = Self::new();
        // Add parameters to context
        for param in func.parameters() {
            ctx.variables.insert(param.name().to_string(), param.param_type().clone());
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

    /// DECY-141: Check if a struct type implements Default (no large arrays)
    /// Structs with arrays > 32 elements don't derive Default due to trait limitations
    fn struct_has_default(&self, struct_name: &str) -> bool {
        if let Some(fields) = self.structs.get(struct_name) {
            // Check if any field has a large array (> 32 elements)
            !fields.iter().any(|(_, field_type)| {
                matches!(
                    field_type,
                    HirType::Array { size: Some(n), .. } if *n > 32
                )
            })
        } else {
            // Unknown struct - assume no Default for safety
            false
        }
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
            // DECY-115: Handle Box<Struct> for heap-allocated structs
            HirType::Box(inner) => {
                if let HirType::Struct(name) = &**inner {
                    name
                } else {
                    return None;
                }
            }
            // DECY-140: Handle Reference<Struct> for borrowed struct access
            HirType::Reference { inner, .. } => {
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
        fields.iter().find(|(name, _)| name == field_name).map(|(_, field_type)| field_type.clone())
    }

    fn is_pointer(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Pointer(_)))
    }

    fn is_option(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Option(_)))
    }

    fn is_vec(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Vec(_)))
    }

    /// Infer the type of an expression based on the context.
    /// Returns None if the type cannot be inferred.
    fn infer_expression_type(&self, expr: &HirExpression) -> Option<HirType> {
        match expr {
            HirExpression::Variable(name) => self.get_type(name).cloned(),
            // DECY-204: Handle literal types for mixed-type arithmetic
            HirExpression::IntLiteral(_) => Some(HirType::Int),
            HirExpression::FloatLiteral(_) => Some(HirType::Double), // C float literals default to double
            HirExpression::CharLiteral(_) => Some(HirType::Char),
            HirExpression::Dereference(inner) => {
                // If inner is *mut T, then *inner is T
                // DECY-123: Also handle Box<T> and &T/&mut T deref → T
                // DECY-151: Also handle Vec<T> (slice representation) deref → T
                match self.infer_expression_type(inner) {
                    Some(HirType::Pointer(pointee_type)) => Some(*pointee_type),
                    Some(HirType::Box(inner_type)) => Some(*inner_type),
                    Some(HirType::Reference { inner: ref_inner, .. }) => Some(*ref_inner),
                    // DECY-151: Vec<T> represents slices, deref gives element type
                    Some(HirType::Vec(elem_type)) => Some(*elem_type),
                    _ => None,
                }
            }
            HirExpression::ArrayIndex { array, index: _ } => {
                // If array is [T; N], *mut T, or &[T], then array[i] is T
                if let Some(array_type) = self.infer_expression_type(array) {
                    match array_type {
                        HirType::Array { element_type, .. } => Some(*element_type),
                        HirType::Pointer(element_type) => Some(*element_type),
                        // DECY-151: Handle slice types (&[T] or &mut [T])
                        // BorrowGenerator uses Reference { inner: Vec(T) } for slices
                        HirType::Reference { inner, .. } => match *inner {
                            HirType::Vec(elem_type) => Some(*elem_type),
                            HirType::Array { element_type, .. } => Some(*element_type),
                            _ => None,
                        },
                        HirType::Vec(elem_type) => Some(*elem_type),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // DECY-123: Handle field access to enable type inference through struct fields
            HirExpression::FieldAccess { object, field } => {
                // Get the struct type from the object expression
                if let Some(obj_type) = self.infer_expression_type(object) {
                    self.get_field_type_from_type(&obj_type, field)
                } else {
                    None
                }
            }
            // DECY-123: Handle pointer field access (ptr->field)
            HirExpression::PointerFieldAccess { pointer, field } => {
                // Get the pointee type (struct) from the pointer expression
                if let Some(ptr_type) = self.infer_expression_type(pointer) {
                    match ptr_type {
                        HirType::Pointer(inner) | HirType::Box(inner) => {
                            self.get_field_type_from_type(&inner, field)
                        }
                        // DECY-123: Handle Reference types (&T, &mut T)
                        HirType::Reference { inner, .. } => {
                            self.get_field_type_from_type(&inner, field)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // DECY-210: Infer type for binary operations
            HirExpression::BinaryOp { left, right, op } => {
                use decy_hir::BinaryOperator;
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);

                // For arithmetic operations, follow C promotion rules
                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide
                    | BinaryOperator::Modulo => {
                        // If either operand is double, result is double
                        if matches!(left_type, Some(HirType::Double))
                            || matches!(right_type, Some(HirType::Double))
                        {
                            return Some(HirType::Double);
                        }
                        // If either operand is float, result is float
                        if matches!(left_type, Some(HirType::Float))
                            || matches!(right_type, Some(HirType::Float))
                        {
                            return Some(HirType::Float);
                        }
                        // Otherwise, result is int (char promotes to int in C)
                        Some(HirType::Int)
                    }
                    // Comparison operations return bool (which we map to int for C compatibility)
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::LogicalAnd
                    | BinaryOperator::LogicalOr => Some(HirType::Int),
                    // Bitwise operations return int
                    BinaryOperator::BitwiseAnd
                    | BinaryOperator::BitwiseOr
                    | BinaryOperator::BitwiseXor
                    | BinaryOperator::LeftShift
                    | BinaryOperator::RightShift => Some(HirType::Int),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// DECY-123: Helper to get field type from a struct type
    fn get_field_type_from_type(&self, obj_type: &HirType, field_name: &str) -> Option<HirType> {
        let struct_name = match obj_type {
            HirType::Struct(name) => name,
            _ => return None,
        };
        let fields = self.structs.get(struct_name)?;
        fields.iter().find(|(name, _)| name == field_name).map(|(_, field_type)| field_type.clone())
    }
}

/// DECY-227: Escape Rust reserved keywords using raw identifiers (r#keyword).
/// C code may use identifiers like `type`, `fn`, `match` that are reserved in Rust.
fn escape_rust_keyword(name: &str) -> String {
    // Rust strict keywords that cannot be used as identifiers without raw syntax
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while",
        // Reserved keywords for future use
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
        "unsized", "virtual", "yield",
    ];

    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else {
        name.to_string()
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
        Self { box_transformer: box_transform::BoxTransformer::new() }
    }

    /// DECY-143: Generate unsafe block with SAFETY comment.
    /// All unsafe blocks should have a comment explaining why the operation is safe.
    fn unsafe_block(code: &str, safety_reason: &str) -> String {
        format!("/* SAFETY: {} */ unsafe {{ {} }}", safety_reason, code)
    }

    /// DECY-143: Generate unsafe statement with SAFETY comment.
    /// For statement-level unsafe blocks (ending with semicolon).
    fn unsafe_stmt(code: &str, safety_reason: &str) -> String {
        format!("// SAFETY: {}\n    unsafe {{ {}; }}", safety_reason, code)
    }

    /// Generate Rust code for a macro definition.
    ///
    /// Transforms C #define macros to Rust const declarations (for object-like macros)
    /// or inline functions (for function-like macros).
    ///
    /// # Supported Macro Types (DECY-098c)
    ///
    /// **Object-like macros** (constants) are fully supported:
    /// - `#define MAX 100` → `const MAX: i32 = 100;`
    /// - `#define PI 3.14159` → `const PI: f64 = 3.14159;`
    /// - `#define GREETING "Hello"` → `const GREETING: &str = "Hello";`
    ///
    /// **Function-like macros** are not yet supported (DECY-098d):
    /// - `#define SQR(x) ((x) * (x))` → Error
    ///
    /// # Type Inference
    ///
    /// Types are automatically inferred from the macro body:
    /// - String literals → `&str`
    /// - Character literals → `char`
    /// - Floating point → `f64`
    /// - Integers (including hex/octal) → `i32`
    ///
    /// # Edge Cases
    ///
    /// - Empty macros generate comments: `#define EMPTY` → `// Empty macro: EMPTY`
    /// - Macro names are preserved exactly (SCREAMING_SNAKE_CASE maintained)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The macro is function-like (not yet implemented)
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::HirMacroDefinition;
    ///
    /// let generator = CodeGenerator::new();
    /// let macro_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    /// let rust_code = generator.generate_macro(&macro_def).unwrap();
    /// assert!(rust_code.contains("const MAX"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Reference
    ///
    /// - K&R §4.11: Macro Substitution
    /// - ISO C99 §6.10.3: Macro replacement
    pub fn generate_macro(
        &self,
        macro_def: &decy_hir::HirMacroDefinition,
    ) -> anyhow::Result<String> {
        if macro_def.is_function_like() {
            // Generate inline function for function-like macros
            return self.generate_function_like_macro(macro_def);
        }

        // Object-like macro (constant)
        let name = macro_def.name();
        let body = macro_def.body();

        // Handle empty macros
        if body.is_empty() {
            return Ok(format!("// Empty macro: {}", name));
        }

        // Infer type from macro body
        let (rust_type, rust_value) = self.infer_macro_type(body)?;

        Ok(format!("const {}: {} = {};", name, rust_type, rust_value))
    }

    /// Generate Rust inline function from function-like macro.
    ///
    /// Transforms C function-like macros to Rust inline functions:
    /// - `#define SQR(x) ((x) * (x))` → `fn sqr(x: i32) -> i32 { x * x }`
    /// - `#define MAX(a, b) ((a) > (b) ? (a) : (b))` → `fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }`
    ///
    /// # Features
    ///
    /// - Converts macro name from SCREAMING_SNAKE_CASE to snake_case
    /// - Infers parameter types (defaults to i32)
    /// - Infers return type from expression
    /// - Adds `#[inline]` attribute for performance
    /// - Transforms ternary operator (? :) to if-else
    /// - Removes unnecessary parentheses
    fn generate_function_like_macro(
        &self,
        macro_def: &decy_hir::HirMacroDefinition,
    ) -> anyhow::Result<String> {
        let name = macro_def.name();
        let params = macro_def.parameters();
        let body = macro_def.body();

        // Convert SCREAMING_SNAKE_CASE to snake_case
        let fn_name = self.convert_to_snake_case(name);

        // Generate parameter list (default to i32 for now)
        let param_list =
            params.iter().map(|p| format!("{}: i32", p)).collect::<Vec<_>>().join(", ");

        // Transform macro body to Rust expression
        let rust_body = self.transform_macro_body(body, params)?;

        // Infer return type from body
        let return_type = self.infer_return_type(body);

        // Generate function
        let result = format!(
            "#[inline]\nfn {}({}) -> {} {{\n    {}\n}}",
            fn_name, param_list, return_type, rust_body
        );

        Ok(result)
    }

    /// Convert SCREAMING_SNAKE_CASE to snake_case.
    fn convert_to_snake_case(&self, name: &str) -> String {
        name.to_lowercase()
    }

    /// Transform C macro body to Rust expression.
    ///
    /// Transformations:
    /// - Remove outer parentheses: ((x) * (x)) → x * x
    /// - Ternary operator: (a) > (b) ? (a) : (b) → if a > b { a } else { b }
    /// - Remove parameter parentheses: (x) → x
    fn transform_macro_body(&self, body: &str, params: &[String]) -> anyhow::Result<String> {
        let mut result = body.to_string();

        // Check for ternary operator
        if result.contains('?') && result.contains(':') {
            result = self.transform_ternary(&result)?;
        } else {
            // Remove unnecessary parentheses around parameters
            for param in params {
                result = result.replace(&format!("({})", param), param);
            }

            // Remove outer parentheses if present
            result = self.remove_outer_parens(&result);

            // Add spaces around operators for readability
            result = self.add_operator_spaces(&result);
        }

        Ok(result)
    }

    /// Transform C ternary operator to Rust if-else.
    ///
    /// Example: ((a)>(b)?(a):(b)) → if a > b { a } else { b }
    fn transform_ternary(&self, expr: &str) -> anyhow::Result<String> {
        // Find the ? and : positions
        let question_pos = expr.find('?').unwrap_or(0);
        let colon_pos = expr.rfind(':').unwrap_or(0);

        if question_pos == 0 || colon_pos == 0 || colon_pos <= question_pos {
            // Malformed ternary, return as-is
            return Ok(expr.to_string());
        }

        // Extract parts
        let condition = expr[..question_pos].trim();
        let true_expr = expr[question_pos + 1..colon_pos].trim();
        let false_expr = expr[colon_pos + 1..].trim();

        // Clean up each part
        let condition = self.remove_outer_parens(condition);
        let condition = self.clean_expression(&condition);
        let true_expr = self.remove_outer_parens(true_expr);
        let true_expr = self.clean_expression(&true_expr);
        let false_expr = self.remove_outer_parens(false_expr);
        let false_expr = self.clean_expression(&false_expr);

        Ok(format!("if {} {{ {} }} else {{ {} }}", condition, true_expr, false_expr))
    }

    /// Remove outer parentheses from expression.
    fn remove_outer_parens(&self, expr: &str) -> String {
        Self::remove_outer_parens_impl(expr)
    }

    /// Implementation of remove_outer_parens (recursive helper).
    fn remove_outer_parens_impl(expr: &str) -> String {
        let trimmed = expr.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            // Check if these are matching outer parens
            let mut depth = 0;
            let chars: Vec<char> = trimmed.chars().collect();
            for (i, ch) in chars.iter().enumerate() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 && i < chars.len() - 1 {
                            // Found closing paren before end, not outer parens
                            return trimmed.to_string();
                        }
                    }
                    _ => {}
                }
            }
            // These are outer parens, remove them
            return Self::remove_outer_parens_impl(&trimmed[1..trimmed.len() - 1]);
        }
        trimmed.to_string()
    }

    /// Clean expression by removing parameter parentheses.
    fn clean_expression(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Handle negation: -(x) → -x (preserve the minus)
        result = result.replace("-(x)", "-x");
        result = result.replace("-(a)", "-a");
        result = result.replace("-(b)", "-b");
        result = result.replace("-(c)", "-c");
        result = result.replace("-(n)", "-n");

        // Remove parentheses around single identifiers (not negated)
        // This is a simplified version - could be enhanced
        result = result.replace("(x)", "x");
        result = result.replace("(a)", "a");
        result = result.replace("(b)", "b");
        result = result.replace("(c)", "c");
        result = result.replace("(n)", "n");

        // Add spaces around operators
        result = self.add_operator_spaces(&result);

        result
    }

    /// Add spaces around operators for readability.
    fn add_operator_spaces(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Add spaces around comparison operators
        result = result.replace(">", " > ");
        result = result.replace("<", " < ");
        result = result.replace("==", " == ");
        result = result.replace("!=", " != ");
        result = result.replace(">=", " >= ");
        result = result.replace("<=", " <= ");

        // Add spaces around logical operators (do this before arithmetic to avoid issues)
        result = result.replace("&&", " && ");
        result = result.replace("||", " || ");

        // Add spaces around arithmetic operators
        result = result.replace("+", " + ");
        // Note: Don't blindly replace "-" as it could be unary minus
        // Only replace if it's not at the start or after a space
        let chars: Vec<char> = result.chars().collect();
        let mut new_result = String::new();
        for (i, ch) in chars.iter().enumerate() {
            if *ch == '-' {
                // Check if this is a binary minus (has non-space before it)
                if i > 0 && !chars[i - 1].is_whitespace() && chars[i - 1] != '(' {
                    new_result.push(' ');
                    new_result.push(*ch);
                    new_result.push(' ');
                } else {
                    // Unary minus, keep as-is
                    new_result.push(*ch);
                }
            } else {
                new_result.push(*ch);
            }
        }
        result = new_result;

        result = result.replace("*", " * ");
        result = result.replace("/", " / ");
        result = result.replace("%", " % ");

        // Clean up multiple spaces
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }

        result.trim().to_string()
    }

    /// Infer return type from macro body.
    ///
    /// Simple heuristic:
    /// - Contains ternary operator (? :) → return type of branches (check for comparison at top level)
    /// - Contains comparison operators at top level (not in ternary) → bool
    /// - Contains logical operators (&&, ||) → bool
    /// - Default → i32
    fn infer_return_type(&self, body: &str) -> String {
        // Check for ternary - return type depends on the branches, not the condition
        if body.contains('?') && body.contains(':') {
            // For ternary, the return type is determined by what's returned, not the condition
            // In most C macros like MAX(a,b), the return type is i32 even though condition is bool
            return "i32".to_string();
        }

        // Check for logical operators (&&, ||) at top level
        if body.contains("&&") || body.contains("||") {
            return "bool".to_string();
        }

        // Check if it's a standalone comparison (no ternary)
        if (body.contains('>') || body.contains('<') || body.contains("==") || body.contains("!="))
            && !body.contains('?')
        {
            // Standalone comparison returns bool
            "bool".to_string()
        } else {
            // Default to i32
            "i32".to_string()
        }
    }

    /// Infer the Rust type and value from a C macro body.
    ///
    /// This function analyzes the macro body string and determines the appropriate
    /// Rust type and formatted value.
    ///
    /// # Type Inference Rules
    ///
    /// - String literals (`"text"`) → `&str`
    /// - Character literals (`'c'`) → `char`
    /// - Floating point (contains `.` or `e`/`E`) → `f64`
    /// - Hexadecimal (`0xFF`) → `i32` (preserves hex format)
    /// - Octal (`0755`) → `i32` (preserves octal format)
    /// - Integers (parseable as i32) → `i32`
    /// - Default (expressions) → `i32`
    ///
    /// # Returns
    ///
    /// Returns a tuple of (rust_type, rust_value) where:
    /// - `rust_type`: The Rust type as a string (e.g., "i32", "&str")
    /// - `rust_value`: The formatted value (e.g., "100", "\"Hello\"")
    ///
    /// # Examples
    ///
    /// ```
    /// # use decy_codegen::CodeGenerator;
    /// let generator = CodeGenerator::new();
    /// // This is a private method, but tested through generate_macro
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn infer_macro_type(&self, body: &str) -> anyhow::Result<(String, String)> {
        let body = body.trim();

        // String literal: "..." → &str
        if body.starts_with('"') && body.ends_with('"') {
            return Ok(("&str".to_string(), body.to_string()));
        }

        // Character literal: '...' → char
        if body.starts_with('\'') && body.ends_with('\'') {
            return Ok(("char".to_string(), body.to_string()));
        }

        // Floating point: contains '.' or 'e'/'E' → f64
        if body.contains('.') || body.contains('e') || body.contains('E') {
            return Ok(("f64".to_string(), body.to_string()));
        }

        // Hexadecimal: 0x... or 0X... → i32 (keep hex format)
        if body.starts_with("0x") || body.starts_with("0X") {
            return Ok(("i32".to_string(), body.to_string()));
        }

        // Octal: 0... → i32
        if body.starts_with('0')
            && body.len() > 1
            && body.chars().nth(1).expect("len>1").is_ascii_digit()
        {
            return Ok(("i32".to_string(), body.to_string()));
        }

        // Try to parse as integer (handles negative numbers too)
        if body.parse::<i32>().is_ok() {
            return Ok(("i32".to_string(), body.to_string()));
        }

        // Default: treat as i32 expression
        Ok(("i32".to_string(), body.to_string()))
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
            HirType::Bool => "bool".to_string(),
            HirType::Int => "i32".to_string(),
            HirType::UnsignedInt => "u32".to_string(), // DECY-158
            HirType::Float => "f32".to_string(),
            HirType::Double => "f64".to_string(),
            HirType::Char => "u8".to_string(),
            HirType::SignedChar => "i8".to_string(), // DECY-250
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
                // DECY-072: Special case for slices: &Vec<T> → &[T]
                if let HirType::Vec(element_type) = &**inner {
                    let element_str = Self::map_type(element_type);
                    if *mutable {
                        format!("&mut [{}]", element_str)
                    } else {
                        format!("&[{}]", element_str)
                    }
                } else if *mutable {
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
            HirType::FunctionPointer { param_types, return_type } => {
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
            HirType::StringLiteral => "&str".to_string(),
            HirType::OwnedString => "String".to_string(),
            HirType::StringReference => "&str".to_string(),
            HirType::Union(_) => {
                // Unions will be transformed to Rust enums
                // For now, return a placeholder
                "/* Union type */".to_string()
            }
            // DECY-172: Preserve typedef names like size_t, ssize_t, ptrdiff_t
            HirType::TypeAlias(name) => name.clone(),
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
            "short" | "short int" => "i16".to_string(),
            "long" | "long int" => "i64".to_string(),
            "long long" | "long long int" => "i64".to_string(),
            "unsigned int" | "unsigned" => "u32".to_string(),
            "unsigned short" | "unsigned short int" => "u16".to_string(),
            "unsigned long" | "unsigned long int" => "u64".to_string(),
            "unsigned long long" | "unsigned long long int" => "u64".to_string(),
            "unsigned char" => "u8".to_string(),
            "signed char" => "i8".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "char" => "u8".to_string(),
            "void" => "()".to_string(),
            // Pointer types
            "char*" | "char *" => "*mut u8".to_string(),
            "int*" | "int *" => "*mut i32".to_string(),
            "void*" | "void *" => "*mut ()".to_string(),
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

}

mod expr_gen;
mod func_gen;
mod stmt_gen;

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

#[cfg(test)]
#[path = "global_variable_codegen_tests.rs"]
mod global_variable_codegen_tests;

#[cfg(test)]
#[path = "coverage_tests.rs"]
mod coverage_tests;

#[cfg(test)]
#[path = "pattern_gen_tests.rs"]
mod pattern_gen_tests;

#[cfg(test)]
#[path = "format_specifier_tests.rs"]
mod format_specifier_tests;

#[cfg(test)]
#[path = "expression_coverage_tests.rs"]
mod expression_coverage_tests;

#[cfg(test)]
#[path = "codegen_coverage_tests.rs"]
mod codegen_coverage_tests;

#[cfg(test)]
#[path = "statement_coverage_tests.rs"]
mod statement_coverage_tests;

#[cfg(test)]
#[path = "expression_target_type_tests.rs"]
mod expression_target_type_tests;

#[cfg(test)]
#[path = "expression_deep_branch_tests.rs"]
mod expression_deep_branch_tests;

#[cfg(test)]
#[path = "box_transform_coverage_tests.rs"]
mod box_transform_coverage_tests;

#[cfg(test)]
#[path = "format_and_sig_tests.rs"]
mod format_and_sig_tests;

#[cfg(test)]
#[path = "expr_stmt_deep_tests.rs"]
mod expr_stmt_deep_tests;

#[cfg(test)]
#[path = "expr_codegen_deep2_tests.rs"]
mod expr_codegen_deep2_tests;

#[cfg(test)]
#[path = "expr_target_deep_tests.rs"]
mod expr_target_deep_tests;

#[cfg(test)]
#[path = "codegen_remaining_tests.rs"]
mod codegen_remaining_tests;

#[cfg(test)]
#[path = "codegen_deep_coverage_tests.rs"]
mod codegen_deep_coverage_tests;

#[cfg(test)]
#[path = "type_context_coverage_tests.rs"]
mod type_context_coverage_tests;
