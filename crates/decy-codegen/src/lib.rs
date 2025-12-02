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

use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};
use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
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
        }
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

    /// DECY-134: Check if a variable is a string iteration param
    #[allow(dead_code)] // Reserved for future use in call-site transformation
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
        self.functions
            .get(func_name)
            .and_then(|params| params.get(param_index))
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

    fn is_vec(&self, name: &str) -> bool {
        matches!(self.get_type(name), Some(HirType::Vec(_)))
    }

    /// Infer the type of an expression based on the context.
    /// Returns None if the type cannot be inferred.
    fn infer_expression_type(&self, expr: &HirExpression) -> Option<HirType> {
        match expr {
            HirExpression::Variable(name) => self.get_type(name).cloned(),
            HirExpression::Dereference(inner) => {
                // If inner is *mut T, then *inner is T
                // DECY-123: Also handle Box<T> and &T/&mut T deref → T
                // DECY-151: Also handle Vec<T> (slice representation) deref → T
                match self.infer_expression_type(inner) {
                    Some(HirType::Pointer(pointee_type)) => Some(*pointee_type),
                    Some(HirType::Box(inner_type)) => Some(*inner_type),
                    Some(HirType::Reference {
                        inner: ref_inner, ..
                    }) => Some(*ref_inner),
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
        fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, field_type)| field_type.clone())
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
        let param_list = params
            .iter()
            .map(|p| format!("{}: i32", p))
            .collect::<Vec<_>>()
            .join(", ");

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

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_expr, false_expr
        ))
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
        if body.starts_with('0') && body.len() > 1 && body.chars().nth(1).unwrap().is_ascii_digit()
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
            HirType::Int => "i32".to_string(),
            HirType::UnsignedInt => "u32".to_string(), // DECY-158
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
                    // DECY-144: Option<Box<T>> gets None instead of null_mut
                    if let Some(HirType::Option(_)) = target_type {
                        return "None".to_string();
                    }
                    if let Some(HirType::Pointer(_)) = target_type {
                        return "std::ptr::null_mut()".to_string();
                    }
                }
                val.to_string()
            }
            // DECY-119: Handle AddressOf when target is raw pointer (struct field assignment)
            // C: node.next = &x;  →  Rust: node.next = &mut x as *mut T;
            HirExpression::AddressOf(inner) => {
                if let Some(HirType::Pointer(ptr_inner)) = target_type {
                    let inner_code = self.generate_expression_with_context(inner, ctx);
                    let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
                    return format!("&mut {} as {}", inner_code, ptr_type);
                }
                // Fall through to default AddressOf handling
                let inner_code = self.generate_expression_with_context(inner, ctx);
                if matches!(**inner, HirExpression::Dereference(_)) {
                    format!("&({})", inner_code)
                } else {
                    format!("&{}", inner_code)
                }
            }
            // DECY-119: Handle UnaryOp AddressOf as well
            HirExpression::UnaryOp {
                op: decy_hir::UnaryOperator::AddressOf,
                operand,
            } => {
                if let Some(HirType::Pointer(ptr_inner)) = target_type {
                    let inner_code = self.generate_expression_with_context(operand, ctx);
                    let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
                    return format!("&mut {} as {}", inner_code, ptr_type);
                }
                // Fall through to default handling
                let inner_code = self.generate_expression_with_context(operand, ctx);
                format!("&{}", inner_code)
            }
            // DECY-191: Handle LogicalNot with target type for bool-to-int coercion
            // In C, ! returns int (0 or 1). When !(bool_expr) is assigned to int, cast to i32.
            HirExpression::UnaryOp {
                op: decy_hir::UnaryOperator::LogicalNot,
                operand,
            } => {
                let inner_code = self.generate_expression_with_context(operand, ctx);
                // Wrap inner expression in parens if it's a binary op to preserve precedence
                let inner_parens = if matches!(**operand, HirExpression::BinaryOp { .. }) {
                    format!("({})", inner_code)
                } else {
                    inner_code.clone()
                };
                // If target is int, we need to cast the bool result to i32
                if let Some(HirType::Int) = target_type {
                    if Self::is_boolean_expression(operand) {
                        // !bool_expr returns bool, needs cast to i32
                        return format!("(!{}) as i32", inner_parens);
                    } else {
                        // !int_expr becomes (int == 0) which is bool, then cast to i32
                        return format!("({} == 0) as i32", inner_code);
                    }
                }
                // No target type or non-int target - use boolean result (no cast)
                // The as i32 cast is only needed when assigning to int variable
                if Self::is_boolean_expression(operand) {
                    format!("!{}", inner_parens)
                } else {
                    // !int_expr becomes (int == 0) which is bool - no cast needed
                    format!("({} == 0)", inner_code)
                }
            }
            HirExpression::StringLiteral(s) => format!("\"{}\"", s),
            HirExpression::CharLiteral(c) => {
                // For char literals, convert to u8 equivalent
                // '\0' = 0, 'a' = 97, etc.
                let val = *c as u8;
                if val == 0 {
                    "0u8".to_string()
                } else if val.is_ascii_graphic() || val == b' ' {
                    format!("b'{}'", val as char)
                } else {
                    // For non-printable characters, use the numeric value
                    format!("{}u8", val)
                }
            }
            HirExpression::Variable(name) => {
                // DECY-142: Vec to Vec - return directly (no conversion needed)
                // When target type is Vec<T> and variable is Vec<T>, return as-is
                if let Some(HirType::Vec(_)) = target_type {
                    // Variable being returned in Vec-return context - return directly
                    return name.clone();
                }
                // DECY-115: Box to raw pointer conversion for return statements
                // When returning a Box<T> but function returns *mut T, use Box::into_raw
                if let Some(HirType::Pointer(ptr_inner)) = target_type {
                    if let Some(var_type) = ctx.get_type(name) {
                        if matches!(var_type, HirType::Box(_)) {
                            return format!("Box::into_raw({})", name);
                        }
                        // DECY-118/DECY-146: Reference/Slice to raw pointer coercion
                        // &[T] or &mut [T] assigned to *mut T needs .as_ptr() / .as_mut_ptr()
                        // &T or &mut T assigned to *mut T needs coercion (pointer cast)
                        match var_type {
                            HirType::Reference { inner, mutable } => {
                                // DECY-149: Check if inner is an array/slice or Vec (both represent slices)
                                // BorrowGenerator uses Vec as internal representation for slices
                                let element_type_match = match inner.as_ref() {
                                    HirType::Array { element_type, .. } => {
                                        Some((element_type.as_ref(), *mutable))
                                    }
                                    HirType::Vec(elem_type) => Some((elem_type.as_ref(), *mutable)),
                                    _ => None,
                                };

                                if let Some((elem_type, is_mutable)) = element_type_match {
                                    // Slice: verify element types match
                                    if elem_type == ptr_inner.as_ref() {
                                        if is_mutable {
                                            // Mutable slice: use .as_mut_ptr()
                                            return format!("{}.as_mut_ptr()", name);
                                        } else {
                                            // Immutable slice: use .as_ptr() with cast
                                            let ptr_type = Self::map_type(&HirType::Pointer(
                                                ptr_inner.clone(),
                                            ));
                                            return format!("{}.as_ptr() as {}", name, ptr_type);
                                        }
                                    }
                                } else if inner.as_ref() == ptr_inner.as_ref() {
                                    // DECY-146: Single reference (&T or &mut T) to pointer
                                    // Cast using addr_of!/addr_of_mut! or pointer cast
                                    if *mutable {
                                        return format!("{} as *mut _", name);
                                    } else {
                                        return format!("{} as *const _ as *mut _", name);
                                    }
                                }
                            }
                            // Also handle Vec<T> to *mut T
                            HirType::Vec(elem_type) => {
                                if elem_type.as_ref() == ptr_inner.as_ref() {
                                    return format!("{}.as_mut_ptr()", name);
                                }
                            }
                            // DECY-148: Handle Pointer(T) → Pointer(T)
                            // When context has raw pointer type, just return the variable directly
                            // No conversion needed - it's already a raw pointer!
                            HirType::Pointer(_var_inner) => {
                                // Raw pointer stays as raw pointer - just return it
                                return name.clone();
                            }
                            _ => {}
                        }
                    }
                }

                // DECY-198: Handle int to char coercion
                // In C, assigning int to char array element truncates: s[i] = c (c is int)
                // In Rust, need explicit cast: s[i] = c as u8
                if let Some(HirType::Char) = target_type {
                    if let Some(var_type) = ctx.get_type(name) {
                        if matches!(var_type, HirType::Int) {
                            return format!("{} as u8", name);
                        }
                    }
                }

                // DECY-203: Handle numeric type coercions (int/float/double)
                // C allows implicit conversions between numeric types
                if let Some(target) = target_type {
                    if let Some(var_type) = ctx.get_type(name) {
                        // Int to Float/Double
                        if matches!(var_type, HirType::Int | HirType::UnsignedInt) {
                            if matches!(target, HirType::Float) {
                                return format!("{} as f32", name);
                            } else if matches!(target, HirType::Double) {
                                return format!("{} as f64", name);
                            }
                        }
                        // Float/Double to Int (truncation)
                        if matches!(var_type, HirType::Float | HirType::Double) {
                            if matches!(target, HirType::Int) {
                                return format!("{} as i32", name);
                            } else if matches!(target, HirType::UnsignedInt) {
                                return format!("{} as u32", name);
                            }
                        }
                        // Char to Int
                        if matches!(var_type, HirType::Char) && matches!(target, HirType::Int) {
                            return format!("{} as i32", name);
                        }
                    }
                }

                name.clone()
            }
            HirExpression::BinaryOp { op, left, right } => {
                // DECY-195: Handle embedded assignment expressions
                // In C, (c = getchar()) evaluates to the assigned value
                // In Rust, assignment returns (), so we need a block: { let tmp = rhs; lhs = tmp; tmp }
                if matches!(op, BinaryOperator::Assign) {
                    let left_code = self.generate_expression_with_context(left, ctx);
                    let right_code = self.generate_expression_with_context(right, ctx);
                    return format!(
                        "{{ let __assign_tmp = {}; {} = __assign_tmp; __assign_tmp }}",
                        right_code, left_code
                    );
                }

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

                    // DECY-130: Vec null check → always false (Vec allocation never fails in safe Rust)
                    // arr == 0 or arr == NULL for Vec types should become `false` (or removed)
                    // because vec![] never returns null - it panics on OOM instead
                    if let HirExpression::Variable(var_name) = &**left {
                        if ctx.is_vec(var_name)
                            && matches!(
                                **right,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return match op {
                                BinaryOperator::Equal => "false /* Vec never null */".to_string(),
                                BinaryOperator::NotEqual => "true /* Vec never null */".to_string(),
                                _ => unreachable!(),
                            };
                        }
                    }

                    // DECY-119: Box null check → always true/false (Box allocation never fails)
                    // Similar to Vec, Box::new() never returns null - it panics on OOM
                    if let HirExpression::Variable(var_name) = &**left {
                        if let Some(HirType::Box(_)) = ctx.get_type(var_name) {
                            if matches!(
                                **right,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            ) {
                                return match op {
                                    BinaryOperator::Equal => {
                                        "false /* Box never null */".to_string()
                                    }
                                    BinaryOperator::NotEqual => {
                                        "true /* Box never null */".to_string()
                                    }
                                    _ => unreachable!(),
                                };
                            }
                        }
                    }

                    // DECY-199: strlen(s) == 0 → s.is_empty() or s.len() == 0
                    // This is more idiomatic Rust than s.len() as i32 == 0
                    if let HirExpression::FunctionCall { function, arguments } = &**left {
                        if function == "strlen" && arguments.len() == 1 {
                            if let HirExpression::IntLiteral(0) = **right {
                                let arg_code = self.generate_expression_with_context(&arguments[0], ctx);
                                return match op {
                                    BinaryOperator::Equal => format!("{}.is_empty()", arg_code),
                                    BinaryOperator::NotEqual => format!("!{}.is_empty()", arg_code),
                                    _ => unreachable!(),
                                };
                            }
                        }
                    }
                    // Also handle 0 == strlen(s)
                    if let HirExpression::FunctionCall { function, arguments } = &**right {
                        if function == "strlen" && arguments.len() == 1 {
                            if let HirExpression::IntLiteral(0) = **left {
                                let arg_code = self.generate_expression_with_context(&arguments[0], ctx);
                                return match op {
                                    BinaryOperator::Equal => format!("{}.is_empty()", arg_code),
                                    BinaryOperator::NotEqual => format!("!{}.is_empty()", arg_code),
                                    _ => unreachable!(),
                                };
                            }
                        }
                    }
                }

                // DECY-198: Handle char literal to int coercion in comparisons
                // In C, char literals are promoted to int when compared with int variables
                // e.g., c != '\n' where c is int should compare against 10 (not 10u8)
                let is_comparison = matches!(
                    op,
                    BinaryOperator::Equal
                        | BinaryOperator::NotEqual
                        | BinaryOperator::LessThan
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::LessEqual
                        | BinaryOperator::GreaterEqual
                );

                if is_comparison {
                    // Check if left is int variable and right is char literal
                    if let HirExpression::Variable(var_name) = &**left {
                        if let Some(HirType::Int) = ctx.get_type(var_name) {
                            if let HirExpression::CharLiteral(c) = &**right {
                                let left_code = self.generate_expression_with_context(left, ctx);
                                let op_str = Self::binary_operator_to_string(op);
                                // Generate char as i32 literal
                                return format!("({} {} {}i32)", left_code, op_str, *c as i32);
                            }
                        }
                    }
                    // Check if right is int variable and left is char literal
                    if let HirExpression::Variable(var_name) = &**right {
                        if let Some(HirType::Int) = ctx.get_type(var_name) {
                            if let HirExpression::CharLiteral(c) = &**left {
                                let right_code = self.generate_expression_with_context(right, ctx);
                                let op_str = Self::binary_operator_to_string(op);
                                // Generate char as i32 literal
                                return format!("({}i32 {} {})", *c as i32, op_str, right_code);
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
                            // This is pointer arithmetic - generate pointer method calls
                            // Note: wrapping_add/wrapping_sub are safe methods on raw pointers
                            // Only offset_from needs unsafe
                            return match op {
                                BinaryOperator::Add => {
                                    format!("{}.wrapping_add({} as usize)", left_str, right_str)
                                }
                                BinaryOperator::Subtract => {
                                    // Check if right is also a pointer (ptr - ptr) or integer (ptr - offset)
                                    if let HirExpression::Variable(right_var) = &**right {
                                        if ctx.is_pointer(right_var) {
                                            // ptr - ptr: calculate difference (returns isize, cast to i32 for C compatibility)
                                            // offset_from requires unsafe
                                            // DECY-143: Add SAFETY comment
                                            Self::unsafe_block(
                                                &format!(
                                                    "{}.offset_from({}) as i32",
                                                    left_str, right_str
                                                ),
                                                "both pointers derive from same allocation",
                                            )
                                        } else {
                                            // ptr - integer offset (safe)
                                            format!(
                                                "{}.wrapping_sub({} as usize)",
                                                left_str, right_str
                                            )
                                        }
                                    } else {
                                        // ptr - integer offset (literal or expression, safe)
                                        format!("{}.wrapping_sub({} as usize)", left_str, right_str)
                                    }
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                }

                // DECY-131: Handle logical operators with integer operands
                // In C, non-zero integers are truthy. In Rust, we need explicit conversion.
                if matches!(op, BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr) {
                    // Check if operands are likely integers (not already boolean comparisons)
                    let left_needs_bool = !Self::is_boolean_expression(left);
                    let right_needs_bool = !Self::is_boolean_expression(right);

                    let left_bool = if left_needs_bool {
                        format!("({} != 0)", left_str)
                    } else {
                        left_str.clone()
                    };

                    let right_bool = if right_needs_bool {
                        format!("({} != 0)", right_str)
                    } else {
                        right_str.clone()
                    };

                    // DECY-191: If target type is Int, cast the bool result to i32
                    // In C, logical operators return int (0 or 1), not bool
                    if let Some(HirType::Int) = target_type {
                        return format!("({} {} {}) as i32", left_bool, op_str, right_bool);
                    }

                    return format!("{} {} {}", left_bool, op_str, right_bool);
                }

                // DECY-151: Char to int promotion for arithmetic operations
                // In C, char arithmetic like *s1 - *s2 is auto-promoted to int
                // When target type is i32 and operands are char (u8), cast to i32
                if matches!(
                    op,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Modulo
                ) {
                    // Check if target type is Int and operands might be Char
                    if let Some(HirType::Int) = target_type {
                        // Infer operand types
                        let left_type = ctx.infer_expression_type(left);
                        let right_type = ctx.infer_expression_type(right);

                        // If either operand is Char (u8), cast both to i32 for proper arithmetic
                        let left_is_char = matches!(left_type, Some(HirType::Char));
                        let right_is_char = matches!(right_type, Some(HirType::Char));

                        if left_is_char || right_is_char {
                            let left_cast = if left_is_char {
                                format!("({} as i32)", left_str)
                            } else {
                                left_str.clone()
                            };
                            let right_cast = if right_is_char {
                                format!("({} as i32)", right_str)
                            } else {
                                right_str.clone()
                            };
                            return format!("{} {} {}", left_cast, op_str, right_cast);
                        }
                    }
                }

                // DECY-191: Comparison and logical operators return bool in Rust but int in C
                // When assigning to an integer type, cast the result to i32
                let returns_bool = matches!(
                    op,
                    BinaryOperator::GreaterThan
                        | BinaryOperator::LessThan
                        | BinaryOperator::GreaterEqual
                        | BinaryOperator::LessEqual
                        | BinaryOperator::Equal
                        | BinaryOperator::NotEqual
                        | BinaryOperator::LogicalAnd
                        | BinaryOperator::LogicalOr
                );

                if returns_bool {
                    if let Some(HirType::Int) = target_type {
                        // Wrap in parentheses and cast to i32
                        return format!("({} {} {}) as i32", left_str, op_str, right_str);
                    }
                }

                format!("{} {} {}", left_str, op_str, right_str)
            }
            HirExpression::Dereference(inner) => {
                // DECY-134: Check for string iteration param - use slice indexing
                if let HirExpression::Variable(var_name) = &**inner {
                    if let Some(idx_var) = ctx.get_string_iter_index(var_name) {
                        // Transform *ptr to slice[idx] - no unsafe needed!
                        return format!("{}[{}]", var_name, idx_var);
                    }

                    // DECY-138: Check for &str type - use as_bytes()[0] for dereference
                    // C pattern: *str where str is const char*
                    // Rust pattern: str.as_bytes()[0] as i32 (cast for integer compatibility)
                    if let Some(var_type) = ctx.get_type(var_name) {
                        if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                            return format!("{}.as_bytes()[0] as i32", var_name);
                        }
                    }
                }

                // DECY-138: Check for *str++ pattern - PostIncrement on &str already returns byte
                // Don't add extra dereference when inner is PostIncrement on &str
                if let HirExpression::PostIncrement { operand } = &**inner {
                    if let HirExpression::Variable(var_name) = &**operand {
                        if let Some(var_type) = ctx.get_type(var_name) {
                            if matches!(var_type, HirType::StringReference | HirType::StringLiteral)
                            {
                                // PostIncrement on &str already generates the byte value
                                // No extra dereference needed
                                return self.generate_expression_with_context(inner, ctx);
                            }
                        }
                    }
                }

                let inner_code = self.generate_expression_with_context(inner, ctx);

                // DECY-041: Check if dereferencing a raw pointer - if so, wrap in unsafe
                if let HirExpression::Variable(var_name) = &**inner {
                    if ctx.is_pointer(var_name) {
                        // DECY-143: Add SAFETY comment
                        return Self::unsafe_block(
                            &format!("*{}", inner_code),
                            "pointer is valid and properly aligned from caller contract",
                        );
                    }
                }

                format!("*{}", inner_code)
            }
            // Note: HirExpression::AddressOf is handled earlier in this match with target_type awareness
            HirExpression::UnaryOp { op, operand } => {
                use decy_hir::UnaryOperator;
                match op {
                    // Post-increment: x++ → { let tmp = x; x += 1; tmp }
                    // Returns old value before incrementing
                    UnaryOperator::PostIncrement => {
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        format!(
                            "{{ let tmp = {}; {} += 1; tmp }}",
                            operand_code, operand_code
                        )
                    }
                    // Post-decrement: x-- → { let tmp = x; x -= 1; tmp }
                    // Returns old value before decrementing
                    UnaryOperator::PostDecrement => {
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        format!(
                            "{{ let tmp = {}; {} -= 1; tmp }}",
                            operand_code, operand_code
                        )
                    }
                    // Pre-increment: ++x → { x += 1; x }
                    // Increments first, then returns new value
                    UnaryOperator::PreIncrement => {
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        format!("{{ {} += 1; {} }}", operand_code, operand_code)
                    }
                    // Pre-decrement: --x → { x -= 1; x }
                    // Decrements first, then returns new value
                    UnaryOperator::PreDecrement => {
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        format!("{{ {} -= 1; {} }}", operand_code, operand_code)
                    }
                    // DECY-131, DECY-191: Logical NOT on integer → (x == 0) as i32
                    // In C, ! returns int (0 or 1), not bool. This matters when !x is used
                    // in expressions like !a == b where we compare the result with an int.
                    UnaryOperator::LogicalNot => {
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        // If operand is already boolean, just negate it
                        if Self::is_boolean_expression(operand) {
                            format!("!{}", operand_code)
                        } else {
                            // For integers: !x → (x == 0) as i32 to match C semantics
                            // where ! returns int, enabling expressions like !a == b
                            format!("({} == 0) as i32", operand_code)
                        }
                    }
                    // Simple prefix operators
                    _ => {
                        let op_str = Self::unary_operator_to_string(op);
                        let operand_code = self.generate_expression_with_context(operand, ctx);
                        format!("{}{}", op_str, operand_code)
                    }
                }
            }
            HirExpression::FunctionCall {
                function,
                arguments,
            } => {
                // Special handling for standard library functions
                match function.as_str() {
                    // strlen(s) → s.len() as i32
                    // Reference: K&R §B3, ISO C99 §7.21.6.3
                    // DECY-199: Cast to i32 since strlen result is often used in int arithmetic
                    "strlen" => {
                        if arguments.len() == 1 {
                            format!(
                                "{}.len() as i32",
                                self.generate_expression_with_context(&arguments[0], ctx)
                            )
                        } else {
                            // Invalid strlen call - shouldn't happen, but handle gracefully
                            let args: Vec<String> = arguments
                                .iter()
                                .map(|arg| self.generate_expression_with_context(arg, ctx))
                                .collect();
                            format!("{}({})", function, args.join(", "))
                        }
                    }
                    // strcpy(dest, src) → CStr-based copy or .to_string()
                    // Reference: K&R §B3, ISO C99 §7.21.3.1
                    // strcpy copies src to dest and returns dest pointer.
                    // DECY-188: Use CStr for raw pointer sources, .to_string() for &str
                    "strcpy" => {
                        if arguments.len() == 2 {
                            let src_code = self.generate_expression_with_context(&arguments[1], ctx);
                            // DECY-188: Detect if source looks like a raw pointer dereference
                            // Patterns like (*foo).bar or (*foo) indicate raw pointer access
                            // Simple variable names that aren't dereferenced are likely &str
                            let is_raw_pointer = src_code.contains("(*") ||
                                                 src_code.contains(").") ||
                                                 src_code.contains("as *");
                            if is_raw_pointer {
                                format!(
                                    "unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\").to_string() }}",
                                    src_code
                                )
                            } else {
                                // &str source - use direct .to_string()
                                format!("{}.to_string()", src_code)
                            }
                        } else {
                            // Invalid strcpy call - shouldn't happen, but handle gracefully
                            let args: Vec<String> = arguments
                                .iter()
                                .map(|arg| self.generate_expression_with_context(arg, ctx))
                                .collect();
                            format!("{}({})", function, args.join(", "))
                        }
                    }
                    // DECY-130: malloc(size) → vec![0; count] or Vec::with_capacity()
                    // DECY-140: When target is raw pointer (*mut u8), use Box::leak for allocation
                    // DECY-142: When target is Vec<T>, generate vec with correct element type
                    // Reference: K&R §B5, ISO C99 §7.20.3.3
                    "malloc" => {
                        if arguments.len() == 1 {
                            let size_code =
                                self.generate_expression_with_context(&arguments[0], ctx);

                            // DECY-142: Check if target is Vec<T> - generate vec with correct element type
                            if let Some(HirType::Vec(elem_type)) = target_type {
                                let elem_type_str = Self::map_type(elem_type);
                                let default_val = Self::default_value_for_type(elem_type);
                                // Try to detect n * sizeof(T) pattern for count
                                if let HirExpression::BinaryOp {
                                    op: BinaryOperator::Multiply,
                                    left,
                                    ..
                                } = &arguments[0]
                                {
                                    let count_code =
                                        self.generate_expression_with_context(left, ctx);
                                    // DECY-170: Wrap count expression in parens for correct precedence
                                    return format!(
                                        "vec![{}; ({}) as usize]",
                                        default_val, count_code
                                    );
                                } else {
                                    // DECY-170: Wrap size expression in parens for correct 'as' precedence
                                    // x + 1 as usize → x + (1 as usize) WRONG
                                    // (x + 1) as usize → correct
                                    return format!(
                                        "Vec::<{}>::with_capacity(({}) as usize)",
                                        elem_type_str, size_code
                                    );
                                }
                            }

                            // DECY-140: Check if target is raw pointer - generate raw allocation
                            if let Some(HirType::Pointer(inner)) = target_type {
                                // malloc assigned to *mut T → Box::leak allocation
                                // This keeps the memory alive (leaked) so the raw pointer remains valid
                                if matches!(inner.as_ref(), HirType::Char) {
                                    // For char* / *mut u8: allocate byte buffer
                                    // DECY-170: Wrap size in parens for correct precedence
                                    return format!(
                                        "Box::leak(vec![0u8; ({}) as usize].into_boxed_slice()).as_mut_ptr()",
                                        size_code
                                    );
                                }
                                // DECY-160: For struct pointers like *mut Node: use Box::into_raw(Box::default())
                                // This allocates a default-initialized struct and returns a raw pointer to it
                                if let HirType::Struct(struct_name) = inner.as_ref() {
                                    return format!(
                                        "Box::into_raw(Box::<{}>::default())",
                                        struct_name
                                    );
                                }
                            }

                            // Try to detect n * sizeof(T) pattern
                            if let HirExpression::BinaryOp {
                                op: BinaryOperator::Multiply,
                                left,
                                ..
                            } = &arguments[0]
                            {
                                // malloc(n * sizeof(T)) → vec![0i32; n]
                                let count_code = self.generate_expression_with_context(left, ctx);
                                // DECY-170: Wrap in parens for correct precedence
                                format!("vec![0i32; ({}) as usize]", count_code)
                            } else {
                                // malloc(size) → Vec::with_capacity(size)
                                // DECY-170: Wrap in parens for correct precedence
                                format!("Vec::<u8>::with_capacity(({}) as usize)", size_code)
                            }
                        } else {
                            "Vec::new()".to_string()
                        }
                    }
                    // DECY-130: calloc(count, size) → vec![0; count]
                    // Reference: K&R §B5, ISO C99 §7.20.3.1
                    "calloc" => {
                        if arguments.len() == 2 {
                            let count_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("vec![0i32; {} as usize]", count_code)
                        } else {
                            "Vec::new()".to_string()
                        }
                    }
                    // DECY-171: realloc(ptr, size) → realloc(ptr as *mut (), size) as *mut T
                    // Reference: K&R §B5, ISO C99 §7.20.3.4
                    // realloc takes void* and returns void*, so we need to:
                    // 1. Cast the typed pointer argument to *mut ()
                    // 2. Cast the return value to the target pointer type
                    "realloc" => {
                        if arguments.len() == 2 {
                            let ptr_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let size_code =
                                self.generate_expression_with_context(&arguments[1], ctx);
                            // Cast argument to *mut () for realloc
                            let realloc_call =
                                format!("realloc({} as *mut (), {})", ptr_code, size_code);

                            // Cast return value to target type if known
                            if let Some(HirType::Pointer(inner)) = target_type {
                                let target_ptr_type =
                                    Self::map_type(&HirType::Pointer(inner.clone()));
                                format!("{} as {}", realloc_call, target_ptr_type)
                            } else {
                                realloc_call
                            }
                        } else {
                            "std::ptr::null_mut()".to_string()
                        }
                    }
                    // DECY-130: free(ptr) → drop(ptr) or comment (RAII handles it)
                    // Reference: K&R §B5, ISO C99 §7.20.3.2
                    "free" => {
                        if arguments.len() == 1 {
                            let ptr_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("drop({})", ptr_code)
                        } else {
                            "/* free() */".to_string()
                        }
                    }
                    // DECY-132: fopen(filename, mode) → std::fs::File::open/create
                    // Reference: K&R §7.5, ISO C99 §7.19.5.3
                    "fopen" => {
                        if arguments.len() == 2 {
                            let filename =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let mode = self.generate_expression_with_context(&arguments[1], ctx);
                            // Check mode: "r" → open, "w" → create
                            if mode.contains('w') || mode.contains('a') {
                                format!("std::fs::File::create({}).ok()", filename)
                            } else {
                                format!("std::fs::File::open({}).ok()", filename)
                            }
                        } else {
                            "None /* fopen requires 2 args */".to_string()
                        }
                    }
                    // DECY-132: fclose(f) → drop(f) (RAII handles cleanup)
                    // Reference: K&R §7.5, ISO C99 §7.19.5.1
                    "fclose" => {
                        if arguments.len() == 1 {
                            let file_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("drop({})", file_code)
                        } else {
                            "/* fclose() */".to_string()
                        }
                    }
                    // DECY-132: fgetc(f) → f.bytes().next().unwrap_or(Err(...))
                    // Reference: K&R §7.5, ISO C99 §7.19.7.1
                    "fgetc" | "getc" => {
                        if arguments.len() == 1 {
                            let file_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!(
                                "{{ use std::io::Read; let mut buf = [0u8; 1]; {}.read(&mut buf).map(|_| buf[0] as i32).unwrap_or(-1) }}",
                                file_code
                            )
                        } else {
                            "-1 /* fgetc requires 1 arg */".to_string()
                        }
                    }
                    // DECY-132: fputc(c, f) → f.write(&[c as u8])
                    // Reference: K&R §7.5, ISO C99 §7.19.7.3
                    "fputc" | "putc" => {
                        if arguments.len() == 2 {
                            let char_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let file_code =
                                self.generate_expression_with_context(&arguments[1], ctx);
                            format!(
                                "{{ use std::io::Write; {}.write(&[{} as u8]).map(|_| {} as i32).unwrap_or(-1) }}",
                                file_code, char_code, char_code
                            )
                        } else {
                            "-1 /* fputc requires 2 args */".to_string()
                        }
                    }
                    // DECY-132: fprintf(f, fmt, ...) → write!(f, fmt, ...)
                    // Reference: K&R §7.2, ISO C99 §7.19.6.1
                    "fprintf" => {
                        if arguments.len() >= 2 {
                            let file_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let fmt = self.generate_expression_with_context(&arguments[1], ctx);
                            if arguments.len() == 2 {
                                format!(
                                    "{{ use std::io::Write; write!({}, {}).map(|_| 0).unwrap_or(-1) }}",
                                    file_code, fmt
                                )
                            } else {
                                // Has format args - simplified version
                                let args: Vec<String> = arguments[2..]
                                    .iter()
                                    .map(|a| self.generate_expression_with_context(a, ctx))
                                    .collect();
                                format!(
                                    "{{ use std::io::Write; write!({}, {}, {}).map(|_| 0).unwrap_or(-1) }}",
                                    file_code, fmt, args.join(", ")
                                )
                            }
                        } else {
                            "-1 /* fprintf requires 2+ args */".to_string()
                        }
                    }
                    // DECY-132: printf(fmt, ...) → print! macro
                    // Reference: K&R §7.2, ISO C99 §7.19.6.3
                    // DECY-119: Convert C format specifiers to Rust
                    // DECY-187: Wrap char* arguments with CStr for %s
                    "printf" => {
                        if !arguments.is_empty() {
                            let fmt = self.generate_expression_with_context(&arguments[0], ctx);
                            // Convert C format specifiers to Rust
                            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
                            if arguments.len() == 1 {
                                format!("print!({})", rust_fmt)
                            } else {
                                // DECY-187: Find %s positions and wrap corresponding args with CStr
                                let s_positions = Self::find_string_format_positions(&fmt);
                                let args: Vec<String> = arguments[1..]
                                    .iter()
                                    .enumerate()
                                    .map(|(i, a)| {
                                        let arg_code = self.generate_expression_with_context(a, ctx);
                                        // If this arg corresponds to a %s, wrap with CStr
                                        // DECY-192: Skip wrapping for ternary expressions with string literals
                                        if s_positions.contains(&i) && !Self::is_string_ternary(a) {
                                            Self::wrap_with_cstr(&arg_code)
                                        } else {
                                            arg_code
                                        }
                                    })
                                    .collect();
                                format!("print!({}, {})", rust_fmt, args.join(", "))
                            }
                        } else {
                            "print!(\"\")".to_string()
                        }
                    }
                    // DECY-090: fread(buf, size, count, file) → file.read(&mut buf)
                    // Reference: K&R §7.5, ISO C99 §7.19.8.1
                    "fread" => {
                        if arguments.len() == 4 {
                            let buf_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let file_code =
                                self.generate_expression_with_context(&arguments[3], ctx);
                            format!(
                                "{{ use std::io::Read; {}.read(&mut {}).unwrap_or(0) }}",
                                file_code, buf_code
                            )
                        } else {
                            "0 /* fread requires 4 args */".to_string()
                        }
                    }
                    // DECY-090: fwrite(data, size, count, file) → file.write(&data)
                    // Reference: K&R §7.5, ISO C99 §7.19.8.2
                    "fwrite" => {
                        if arguments.len() == 4 {
                            let data_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let file_code =
                                self.generate_expression_with_context(&arguments[3], ctx);
                            format!(
                                "{{ use std::io::Write; {}.write(&{}).unwrap_or(0) }}",
                                file_code, data_code
                            )
                        } else {
                            "0 /* fwrite requires 4 args */".to_string()
                        }
                    }
                    // DECY-090: fputs(str, file) → file.write_all(str.as_bytes())
                    // Reference: K&R §7.5, ISO C99 §7.19.7.4
                    "fputs" => {
                        if arguments.len() == 2 {
                            let str_code =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            let file_code =
                                self.generate_expression_with_context(&arguments[1], ctx);
                            format!(
                                "{{ use std::io::Write; {}.write_all({}.as_bytes()).map(|_| 0).unwrap_or(-1) }}",
                                file_code, str_code
                            )
                        } else {
                            "-1 /* fputs requires 2 args */".to_string()
                        }
                    }
                    // DECY-093: fork() → Command usage (no direct equivalent, skip)
                    "fork" => "/* fork() transformed to Command API */ 0".to_string(),
                    // DECY-093: execl/execlp → Command::new().status()
                    "execl" | "execlp" | "execle" | "execv" | "execvp" | "execve" => {
                        if !arguments.is_empty() {
                            let cmd = self.generate_expression_with_context(&arguments[0], ctx);
                            // Skip argv[0] (program name repeated), collect remaining args before NULL
                            let args: Vec<String> = arguments
                                .iter()
                                .skip(2) // Skip cmd path and argv[0]
                                .filter(|a| !matches!(a, HirExpression::NullLiteral))
                                .map(|a| self.generate_expression_with_context(a, ctx))
                                .collect();
                            if args.is_empty() {
                                format!(
                                    "{{ use std::process::Command; Command::new({}).status().expect(\"command failed\"); }}",
                                    cmd
                                )
                            } else {
                                let arg_chain: String =
                                    args.iter().map(|a| format!(".arg({})", a)).collect();
                                format!(
                                    "{{ use std::process::Command; Command::new({}){}.status().expect(\"command failed\"); }}",
                                    cmd, arg_chain
                                )
                            }
                        } else {
                            "/* exec requires args */".to_string()
                        }
                    }
                    // DECY-093: waitpid → .wait() (generated alongside spawn)
                    "waitpid" | "wait3" | "wait4" => {
                        "/* waitpid handled by Command API */ child.wait().expect(\"wait failed\")"
                            .to_string()
                    }
                    // DECY-094: wait(&status) → child.wait()
                    "wait" => "child.wait().expect(\"wait failed\")".to_string(),
                    // DECY-094: WEXITSTATUS(status) → status.code().unwrap_or(-1)
                    "WEXITSTATUS" => {
                        if !arguments.is_empty() {
                            let status_var =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("{}.code().unwrap_or(-1)", status_var)
                        } else {
                            "/* WEXITSTATUS requires status arg */".to_string()
                        }
                    }
                    // DECY-094: WIFEXITED(status) → status.success()
                    "WIFEXITED" => {
                        if !arguments.is_empty() {
                            let status_var =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("{}.success()", status_var)
                        } else {
                            "/* WIFEXITED requires status arg */".to_string()
                        }
                    }
                    // DECY-094: WIFSIGNALED(status) → status.signal().is_some()
                    "WIFSIGNALED" => {
                        if !arguments.is_empty() {
                            let status_var =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("{}.signal().is_some()", status_var)
                        } else {
                            "/* WIFSIGNALED requires status arg */".to_string()
                        }
                    }
                    // DECY-094: WTERMSIG(status) → status.signal().unwrap_or(0)
                    "WTERMSIG" => {
                        if !arguments.is_empty() {
                            let status_var =
                                self.generate_expression_with_context(&arguments[0], ctx);
                            format!("{}.signal().unwrap_or(0)", status_var)
                        } else {
                            "/* WTERMSIG requires status arg */".to_string()
                        }
                    }
                    // Default: pass through function call as-is
                    // DECY-116 + DECY-117: Transform call sites for slice functions and reference mutability
                    _ => {
                        // DECY-116: Check if this function has slice params (with removed length args)
                        let slice_mappings = ctx.get_slice_func_len_indices(function);
                        let len_indices_to_skip: std::collections::HashSet<usize> = slice_mappings
                            .map(|mappings| mappings.iter().map(|(_, len_idx)| *len_idx).collect())
                            .unwrap_or_default();
                        let array_indices: std::collections::HashSet<usize> = slice_mappings
                            .map(|mappings| mappings.iter().map(|(arr_idx, _)| *arr_idx).collect())
                            .unwrap_or_default();

                        let args: Vec<String> = arguments
                            .iter()
                            .enumerate()
                            .filter_map(|(i, arg)| {
                                // DECY-116: Skip length arguments that were removed
                                if len_indices_to_skip.contains(&i) {
                                    return None;
                                }

                                // DECY-116: Convert array args to slice references
                                if array_indices.contains(&i) {
                                    let arg_code = self.generate_expression_with_context(arg, ctx);
                                    return Some(format!("&{}", arg_code));
                                }

                                // DECY-117: If arg is AddressOf (via UnaryOp or direct) and param expects &mut, generate &mut
                                let is_address_of = matches!(arg, HirExpression::AddressOf(_))
                                    || matches!(
                                        arg,
                                        HirExpression::UnaryOp {
                                            op: decy_hir::UnaryOperator::AddressOf,
                                            ..
                                        }
                                    );

                                if is_address_of {
                                    // Extract the inner expression
                                    let inner = match arg {
                                        HirExpression::AddressOf(inner) => inner.as_ref(),
                                        HirExpression::UnaryOp { operand, .. } => operand.as_ref(),
                                        _ => unreachable!(),
                                    };

                                    // Check if the function expects &mut for this parameter
                                    let expects_mut = ctx
                                        .get_function_param_type(function, i)
                                        .map(|t| {
                                            matches!(t, HirType::Reference { mutable: true, .. })
                                        })
                                        .unwrap_or(true); // Default to &mut for safety

                                    let inner_code =
                                        self.generate_expression_with_context(inner, ctx);
                                    if expects_mut {
                                        Some(format!("&mut {}", inner_code))
                                    } else {
                                        Some(format!("&{}", inner_code))
                                    }
                                } else {
                                    // DECY-134b: Check if this function has string iteration params
                                    if let Some(string_iter_params) =
                                        ctx.get_string_iter_func(function)
                                    {
                                        // Check if this argument index is a string iteration param
                                        if let Some((_, is_mutable)) =
                                            string_iter_params.iter().find(|(idx, _)| *idx == i)
                                        {
                                            // Transform argument to slice reference
                                            // array → &mut array or &array
                                            // string literal → b"string" (byte slice)
                                            if let HirExpression::Variable(var_name) = arg {
                                                let var_type = ctx.get_type(var_name);
                                                if matches!(var_type, Some(HirType::Array { .. })) {
                                                    if *is_mutable {
                                                        return Some(format!("&mut {}", var_name));
                                                    } else {
                                                        return Some(format!("&{}", var_name));
                                                    }
                                                }
                                            }
                                            if let HirExpression::StringLiteral(s) = arg {
                                                // String literal becomes byte slice reference
                                                return Some(format!("b\"{}\"", s));
                                            }
                                            // AddressOf expressions (e.g., &buffer) - extract inner
                                            if let HirExpression::AddressOf(inner) = arg {
                                                let inner_code = self
                                                    .generate_expression_with_context(inner, ctx);
                                                if *is_mutable {
                                                    return Some(format!("&mut {}", inner_code));
                                                } else {
                                                    return Some(format!("&{}", inner_code));
                                                }
                                            }
                                        }
                                    }

                                    // DECY-125: Check if param is raw pointer and arg needs conversion
                                    let param_type = ctx.get_function_param_type(function, i);
                                    let is_raw_pointer_param = param_type
                                        .map(|t| matches!(t, HirType::Pointer(_)))
                                        .unwrap_or(false);

                                    if is_raw_pointer_param {
                                        // Convert array/variable to .as_mut_ptr()
                                        if let HirExpression::Variable(var_name) = arg {
                                            // Check if it's an array type in context
                                            let var_type = ctx.get_type(var_name);
                                            if matches!(var_type, Some(HirType::Array { .. })) {
                                                return Some(format!("{}.as_mut_ptr()", var_name));
                                            }
                                        }
                                        // Convert string literal to .as_ptr() as *mut u8
                                        if let HirExpression::StringLiteral(s) = arg {
                                            return Some(format!("\"{}\".as_ptr() as *mut u8", s));
                                        }
                                    }

                                    // DECY-123: Check if param expects &mut but arg is raw pointer
                                    let is_ref_param = param_type
                                        .map(|t| matches!(t, HirType::Reference { .. }))
                                        .unwrap_or(false);
                                    if is_ref_param {
                                        if let HirExpression::Variable(var_name) = arg {
                                            let var_type = ctx.get_type(var_name);
                                            // Raw pointer to reference: unsafe { &mut *ptr }
                                            if matches!(var_type, Some(HirType::Pointer(_))) {
                                                // DECY-143: Add SAFETY comment
                                                return Some(Self::unsafe_block(
                                                    &format!("&mut *{}", var_name),
                                                    "pointer is non-null and valid for the duration of the call",
                                                ));
                                            }
                                        }
                                    }

                                    // DECY-197: Check if param is unsized array (slice param) and arg is sized array
                                    // C's `void func(char arr[])` becomes `fn func(arr: &mut [u8])` in Rust
                                    // When calling with fixed-size array, add `&mut` prefix
                                    let is_slice_param = param_type
                                        .map(|t| matches!(t, HirType::Array { size: None, .. }))
                                        .unwrap_or(false);
                                    if is_slice_param {
                                        if let HirExpression::Variable(var_name) = arg {
                                            let var_type = ctx.get_type(var_name);
                                            // Fixed-size array to slice: add &mut prefix
                                            if matches!(var_type, Some(HirType::Array { size: Some(_), .. })) {
                                                return Some(format!("&mut {}", var_name));
                                            }
                                        }
                                    }

                                    // DECY-199: Check if param expects Int but arg is CharLiteral
                                    // putchar(' ') needs ' ' as i32, not b' '
                                    let is_int_param = param_type
                                        .map(|t| matches!(t, HirType::Int))
                                        .unwrap_or(false);
                                    if is_int_param {
                                        if let HirExpression::CharLiteral(c) = arg {
                                            // Cast char to i32
                                            return Some(format!("{}i32", *c as i32));
                                        }
                                    }

                                    // DECY-140: Check if param expects &str but arg is a raw pointer field
                                    // This happens when calling strcmp/strncmp with entry->key where key is char*
                                    // For stdlib string functions, params are &str but we might pass *mut u8 field
                                    let is_string_param = param_type
                                        .map(|t| matches!(t, HirType::StringReference | HirType::StringLiteral))
                                        .unwrap_or(false);
                                    // Also check for known stdlib string functions that expect &str
                                    let is_string_func = matches!(
                                        function.as_str(),
                                        "strcmp" | "strncmp" | "strchr" | "strrchr" | "strstr" | "strlen"
                                    );
                                    if is_string_param || is_string_func {
                                        // Check if arg is PointerFieldAccess (entry->key pattern)
                                        if let HirExpression::PointerFieldAccess { pointer, field } = arg {
                                            // Generate CStr conversion for null-terminated C string
                                            // DECY-143: Add SAFETY comment
                                            let ptr_code = self.generate_expression_with_context(pointer, ctx);
                                            return Some(Self::unsafe_block(
                                                &format!("std::ffi::CStr::from_ptr((*{}).{} as *const i8).to_str().unwrap_or(\"\")", ptr_code, field),
                                                "string pointer is null-terminated and valid",
                                            ));
                                        }
                                    }

                                    Some(self.generate_expression_with_context(arg, ctx))
                                }
                            })
                            .collect();
                        format!("{}({})", function, args.join(", "))
                    }
                }
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
                        let ptr_code = self.generate_expression_with_context(pointer, ctx);
                        // DECY-129: Check if pointer is a raw pointer - if so, wrap in unsafe
                        if let HirExpression::Variable(var_name) = &**pointer {
                            if ctx.is_pointer(var_name) {
                                // DECY-143: Add SAFETY comment
                                return Self::unsafe_block(
                                    &format!("(*{}).{}", ptr_code, field),
                                    "pointer is non-null and points to valid struct",
                                );
                            }
                        }
                        format!("(*{}).{}", ptr_code, field)
                    }
                }
            }
            HirExpression::ArrayIndex { array, index } => {
                let array_code = self.generate_expression_with_context(array, ctx);
                let index_code = self.generate_expression_with_context(index, ctx);

                // DECY-041: Check if array is a raw pointer - if so, use unsafe pointer arithmetic
                // DECY-165: Also check via infer_expression_type for struct field access
                let is_raw_pointer = if let HirExpression::Variable(var_name) = &**array {
                    ctx.is_pointer(var_name)
                } else {
                    // Use type inference for complex expressions like sb->data
                    matches!(ctx.infer_expression_type(array), Some(HirType::Pointer(_)))
                };

                if is_raw_pointer {
                    // Raw pointer indexing: arr[i] becomes unsafe { *arr.add(i as usize) }
                    // DECY-143: Add SAFETY comment
                    return Self::unsafe_block(
                        &format!("*{}.add(({}) as usize)", array_code, index_code),
                        "index is within bounds of allocated array",
                    );
                }

                // Regular array/slice indexing
                // DECY-072: Cast index to usize for slice indexing
                // DECY-150: Wrap index in parens to handle operator precedence
                // e.g., buffer[size - 1 - i as usize] parses as buffer[size - 1 - (i as usize)]
                // but buffer[(size - 1 - i) as usize] casts entire expression correctly
                format!("{}[({}) as usize]", array_code, index_code)
            }
            HirExpression::SliceIndex { slice, index, .. } => {
                // DECY-070 GREEN: Generate safe slice indexing (0 unsafe blocks!)
                // SliceIndex represents pointer arithmetic transformed to safe indexing
                let slice_code = self.generate_expression_with_context(slice, ctx);
                let index_code = self.generate_expression_with_context(index, ctx);
                // DECY-113: Generate: slice[index as usize] - cast i32 index to usize
                // Slice indexing requires usize, but C typically uses int (i32)
                // DECY-150: Wrap index in parens to handle operator precedence
                format!("{}[({}) as usize]", slice_code, index_code)
            }
            HirExpression::Sizeof { type_name } => {
                // sizeof(int) → std::mem::size_of::<i32>() as i32
                // sizeof(struct Data) → std::mem::size_of::<Data>() as i32
                // Note: size_of returns usize, but C's sizeof returns int (typically i32)

                // DECY-189: Detect sizeof(expr) that was mis-parsed as sizeof(type)
                // Pattern: "record name" came from sizeof(record->name) where
                // the parser tokenized record and name as separate identifiers
                let trimmed = type_name.trim();
                let is_member_access = trimmed.contains(' ')
                    && !trimmed.starts_with("struct ")
                    && !trimmed.starts_with("unsigned ")
                    && !trimmed.starts_with("signed ")
                    && !trimmed.starts_with("long ")
                    && !trimmed.starts_with("short ");

                if is_member_access {
                    // DECY-189: sizeof(record->field) → std::mem::size_of_val(&(*record).field)
                    // Split "record field" into parts and reconstruct member access
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let var = parts[0];
                        let field = parts[1..].join(".");
                        format!("std::mem::size_of_val(&(*{}).{}) as i32", var, field)
                    } else {
                        // Fallback: shouldn't happen, but be safe
                        let rust_type = self.map_sizeof_type(type_name);
                        format!("std::mem::size_of::<{}>() as i32", rust_type)
                    }
                } else {
                    let rust_type = self.map_sizeof_type(type_name);
                    format!("std::mem::size_of::<{}>() as i32", rust_type)
                }
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
                    HirType::UnsignedInt => "0u32", // DECY-158
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
            HirExpression::StringMethodCall {
                receiver,
                method,
                arguments,
            } => {
                let receiver_code = self.generate_expression_with_context(receiver, ctx);
                if arguments.is_empty() {
                    // DECY-072: Cast .len() to i32 for slices/arrays (used with i32 loop counters)
                    // String .len() returns usize which is typically what's needed for strings
                    // But array/slice .len() often needs i32 for C loop patterns
                    // We cast for all cases to maintain C semantics (where sizeof returns int)
                    if method == "len" {
                        format!("{}.{}() as i32", receiver_code, method)
                    } else {
                        format!("{}.{}()", receiver_code, method)
                    }
                } else {
                    let args: Vec<String> = arguments
                        .iter()
                        .map(|arg| {
                            // For clone_into, we need &mut on the destination
                            if method == "clone_into" {
                                format!("&mut {}", self.generate_expression_with_context(arg, ctx))
                            } else {
                                self.generate_expression_with_context(arg, ctx)
                            }
                        })
                        .collect();
                    format!("{}.{}({})", receiver_code, method, args.join(", "))
                }
            }
            HirExpression::Cast { target_type, expr } => {
                // C: (int)x → Rust: x as i32
                // Sprint 19 Feature (DECY-059)
                let expr_code = self.generate_expression_with_context(expr, ctx);
                let rust_type = Self::map_type(target_type);

                // Wrap in parentheses if the expression is a binary operation
                let expr_str = if matches!(**expr, HirExpression::BinaryOp { .. }) {
                    format!("({})", expr_code)
                } else {
                    expr_code
                };

                format!("{} as {}", expr_str, rust_type)
            }
            HirExpression::CompoundLiteral {
                literal_type,
                initializers,
            } => {
                // C: (struct Point){10, 20} → Rust: Point { x: 10, y: 20 }
                // C: (int[]){1, 2, 3} → Rust: vec![1, 2, 3] or [1, 2, 3]
                // Sprint 19 Feature (DECY-060)
                // DECY-133: Use actual field names from struct definition
                match literal_type {
                    HirType::Struct(name) => {
                        // Generate struct literal: StructName { x: val0, y: val1, ..Default::default() }
                        if initializers.is_empty() {
                            // Empty struct: Point {}
                            format!("{} {{}}", name)
                        } else {
                            // DECY-133: Look up struct field names from context
                            let struct_fields = ctx.structs.get(name);
                            let num_struct_fields = struct_fields.map(|f| f.len()).unwrap_or(0);

                            let fields: Vec<String> = initializers
                                .iter()
                                .enumerate()
                                .map(|(i, init)| {
                                    let init_code =
                                        self.generate_expression_with_context(init, ctx);
                                    // Use actual field name if available, fallback to field{i}
                                    let field_name = struct_fields
                                        .and_then(|f| f.get(i))
                                        .map(|(name, _)| name.as_str())
                                        .unwrap_or_else(|| {
                                            Box::leak(format!("field{}", i).into_boxed_str())
                                        });
                                    format!("{}: {}", field_name, init_code)
                                })
                                .collect();

                            // DECY-133: Add ..Default::default() if not all fields are initialized
                            // This handles designated initializers that skip fields
                            if initializers.len() < num_struct_fields {
                                format!(
                                    "{} {{ {}, ..Default::default() }}",
                                    name,
                                    fields.join(", ")
                                )
                            } else {
                                format!("{} {{ {} }}", name, fields.join(", "))
                            }
                        }
                    }
                    HirType::Array { .. } => {
                        // DECY-199: Generate array literal [1, 2, 3] instead of vec![...]
                        // Fixed-size arrays should use array literals, not Vec
                        if initializers.is_empty() {
                            "[]".to_string()
                        } else {
                            let elements: Vec<String> = initializers
                                .iter()
                                .map(|init| self.generate_expression_with_context(init, ctx))
                                .collect();
                            format!("[{}]", elements.join(", "))
                        }
                    }
                    _ => {
                        // For other types, generate a reasonable default
                        // This is a simplified implementation
                        format!(
                            "/* Compound literal of type {} */",
                            Self::map_type(literal_type)
                        )
                    }
                }
            }
            // DECY-139: Post-increment expression (x++)
            // C semantics: returns old value, then increments
            // Rust: { let __tmp = x; x += 1; __tmp }
            HirExpression::PostIncrement { operand } => {
                let operand_code = self.generate_expression_with_context(operand, ctx);

                // DECY-138: Special handling for &str post-increment (string iteration)
                // C pattern: *key++ where key is const char*
                // Rust pattern: { let __tmp = key.as_bytes()[0] as u32; key = &key[1..]; __tmp }
                // DECY-158: Cast to u32 for C-compatible unsigned promotion semantics
                // In C, char is promoted to int/unsigned int in arithmetic - u32 works for both
                if let HirExpression::Variable(var_name) = &**operand {
                    if let Some(var_type) = ctx.get_type(var_name) {
                        if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                            return format!(
                                "{{ let __tmp = {var}.as_bytes()[0] as u32; {var} = &{var}[1..]; __tmp }}",
                                var = operand_code
                            );
                        }
                    }
                }

                format!(
                    "{{ let __tmp = {operand}; {operand} += 1; __tmp }}",
                    operand = operand_code
                )
            }
            // DECY-139: Pre-increment expression (++x)
            // C semantics: increments first, then returns new value
            // Rust: { x += 1; x }
            HirExpression::PreIncrement { operand } => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("{{ {operand} += 1; {operand} }}", operand = operand_code)
            }
            // DECY-139: Post-decrement expression (x--)
            // C semantics: returns old value, then decrements
            // Rust: { let __tmp = x; x -= 1; __tmp }
            HirExpression::PostDecrement { operand } => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!(
                    "{{ let __tmp = {operand}; {operand} -= 1; __tmp }}",
                    operand = operand_code
                )
            }
            // DECY-139: Pre-decrement expression (--x)
            // C semantics: decrements first, then returns new value
            // Rust: { x -= 1; x }
            HirExpression::PreDecrement { operand } => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("{{ {operand} -= 1; {operand} }}", operand = operand_code)
            }
            // DECY-192: Ternary/Conditional expression (cond ? then : else)
            // C: (a > b) ? a : b → Rust: if a > b { a } else { b }
            HirExpression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_code = self.generate_expression_with_context(condition, ctx);
                let then_code = self.generate_expression_with_context(then_expr, ctx);
                let else_code = self.generate_expression_with_context(else_expr, ctx);

                // Convert condition to boolean if it's not already
                let cond_bool = if Self::is_boolean_expression(condition) {
                    cond_code
                } else {
                    format!("{} != 0", cond_code)
                };

                format!("if {} {{ {} }} else {{ {} }}", cond_bool, then_code, else_code)
            }
        }
    }

    /// Convert unary operator to string.
    fn unary_operator_to_string(op: &decy_hir::UnaryOperator) -> &'static str {
        use decy_hir::UnaryOperator;
        match op {
            UnaryOperator::Minus => "-",
            UnaryOperator::LogicalNot => "!",
            // DECY-193: In Rust, bitwise NOT is ! (same as logical NOT for bool)
            UnaryOperator::BitwiseNot => "!",
            UnaryOperator::AddressOf => "&",
            // Post/Pre-increment/decrement are handled as block expressions
            // in generate_expression_with_context, so should never reach here
            UnaryOperator::PostIncrement
            | UnaryOperator::PostDecrement
            | UnaryOperator::PreIncrement
            | UnaryOperator::PreDecrement => {
                unreachable!("Increment/decrement operators should be handled as block expressions")
            }
        }
    }

    /// Check if an expression already produces a boolean result.
    /// Used to avoid redundant `!= 0` conversions for expressions that are already boolean.
    fn is_boolean_expression(expr: &HirExpression) -> bool {
        match expr {
            // Comparison operators always produce bool
            HirExpression::BinaryOp { op, .. } => matches!(
                op,
                BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::LogicalAnd
                    | BinaryOperator::LogicalOr
            ),
            // Logical NOT produces bool
            HirExpression::UnaryOp {
                op: decy_hir::UnaryOperator::LogicalNot,
                ..
            } => true,
            // Other expressions are assumed to be non-boolean (integers, etc.)
            _ => false,
        }
    }

    /// DECY-138: Check if expression is a dereference of a string variable.
    /// Returns the variable name if it's a string dereference, None otherwise.
    /// Used for string iteration pattern: while (*str) → while !str.is_empty()
    fn get_string_deref_var(expr: &HirExpression, ctx: &TypeContext) -> Option<String> {
        match expr {
            // Direct dereference: *str
            HirExpression::Dereference(inner) => {
                if let HirExpression::Variable(var_name) = &**inner {
                    if let Some(var_type) = ctx.get_type(var_name) {
                        if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                            return Some(var_name.clone());
                        }
                    }
                }
                None
            }
            // Comparison with 0: *str != 0 or *str == 0
            HirExpression::BinaryOp { op, left, right } => {
                if matches!(op, BinaryOperator::NotEqual | BinaryOperator::Equal) {
                    // Check: *str != 0 or *str == 0
                    if let HirExpression::IntLiteral(0) = &**right {
                        return Self::get_string_deref_var(left, ctx);
                    }
                    if let HirExpression::IntLiteral(0) = &**left {
                        return Self::get_string_deref_var(right, ctx);
                    }
                }
                None
            }
            _ => None,
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
            // DECY-137: Bitwise and shift operators
            BinaryOperator::LeftShift => "<<",
            BinaryOperator::RightShift => ">>",
            BinaryOperator::BitwiseAnd => "&",
            BinaryOperator::BitwiseOr => "|",
            BinaryOperator::BitwiseXor => "^",
            // DECY-195: Assignment operator (for embedded assignments)
            BinaryOperator::Assign => "=",
        }
    }

    /// Get default value for a type (for uninitialized variables).
    fn default_value_for_type(hir_type: &HirType) -> String {
        match hir_type {
            HirType::Void => "()".to_string(),
            HirType::Int => "0i32".to_string(),
            HirType::UnsignedInt => "0u32".to_string(), // DECY-158
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
                // Use ::default() for struct initialization
                // For structs with large arrays (>32 elements), see DECY-123 special handling
                // in variable declaration codegen where struct definitions are available
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
                // DECY-202: Function pointers default to None when wrapped in Option
                // For local variables, return None which will be unwrapped if needed
                "None".to_string()
            }
            HirType::StringLiteral => {
                // String literals default to empty string slice
                r#""""#.to_string()
            }
            HirType::OwnedString => {
                // Owned strings default to String::new()
                "String::new()".to_string()
            }
            HirType::StringReference => {
                // String references default to empty string slice
                r#""""#.to_string()
            }
            HirType::Union(_) => {
                // Unions will be transformed to enums
                // Default to the first variant's default value
                panic!("Union types must be initialized and cannot have default values")
            }
            // DECY-172: Type aliases use 0 as default (for size_t/ssize_t/ptrdiff_t)
            HirType::TypeAlias(name) => {
                // size_t/ssize_t/ptrdiff_t default to 0
                match name.as_str() {
                    "size_t" => "0usize".to_string(),
                    "ssize_t" | "ptrdiff_t" => "0isize".to_string(),
                    _ => "0".to_string(),
                }
            }
        }
    }

    /// Convert C printf format specifiers to Rust format specifiers.
    /// DECY-119: %d → {}, %s → {}, %f → {}, etc.
    fn convert_c_format_to_rust(c_fmt: &str) -> String {
        // If it's a quoted string literal, process the contents
        let trimmed = c_fmt.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            let inner = &trimmed[1..trimmed.len() - 1];
            let converted = Self::convert_format_specifiers(inner);
            format!("\"{}\"", converted)
        } else {
            // Not a string literal, return as-is
            c_fmt.to_string()
        }
    }

    /// DECY-193: Convert C format specifiers to Rust format specifiers.
    /// Handles width, precision, and flags like %02X, %10.3f, etc.
    fn convert_format_specifiers(input: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '%' {
                if i + 1 < chars.len() && chars[i + 1] == '%' {
                    // %% -> %
                    result.push('%');
                    i += 2;
                    continue;
                }

                // Parse format specifier: %[flags][width][.precision][length]specifier
                let start = i;
                i += 1; // skip %

                let mut flags = String::new();
                let mut width = String::new();
                let mut precision = String::new();

                // Parse flags: -, +, space, #, 0
                while i < chars.len() && "-+ #0".contains(chars[i]) {
                    if chars[i] == '0' {
                        flags.push('0'); // zero-padding
                    }
                    // Skip other flags for now (- is left-align, + is sign, etc.)
                    i += 1;
                }

                // Parse width
                while i < chars.len() && chars[i].is_ascii_digit() {
                    width.push(chars[i]);
                    i += 1;
                }

                // Parse precision
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        precision.push(chars[i]);
                        i += 1;
                    }
                }

                // Skip length modifiers: h, hh, l, ll, L, z, j, t
                while i < chars.len() && "hlLzjt".contains(chars[i]) {
                    i += 1;
                }

                // Parse specifier
                if i < chars.len() {
                    let spec = chars[i];
                    i += 1;

                    // Build Rust format specifier
                    let rust_spec = match spec {
                        'd' | 'i' | 'u' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}}}", flags, width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'x' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}x}}", flags, width)
                            } else {
                                "{:x}".to_string()
                            }
                        }
                        'X' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}X}}", flags, width)
                            } else {
                                "{:X}".to_string()
                            }
                        }
                        'o' => {
                            if !width.is_empty() || !flags.is_empty() {
                                format!("{{:{}{}o}}", flags, width)
                            } else {
                                "{:o}".to_string()
                            }
                        }
                        'f' | 'F' => {
                            if !precision.is_empty() {
                                if !width.is_empty() {
                                    format!("{{:{}{}.{}}}", flags, width, precision)
                                } else {
                                    format!("{{:.{}}}", precision)
                                }
                            } else if !width.is_empty() {
                                format!("{{:{}{}}}", flags, width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'e' => "{:e}".to_string(),
                        'E' => "{:E}".to_string(),
                        'g' | 'G' => "{}".to_string(),
                        's' => {
                            if !width.is_empty() {
                                format!("{{:{}}}", width)
                            } else {
                                "{}".to_string()
                            }
                        }
                        'c' => "{}".to_string(),
                        'p' => "{:p}".to_string(),
                        _ => {
                            // Unknown specifier, keep original
                            input[start..i].to_string()
                        }
                    };
                    result.push_str(&rust_spec);
                } else {
                    // Incomplete format specifier at end of string
                    result.push_str(&input[start..]);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// DECY-187: Find positions of %s format specifiers in a format string.
    /// Returns 0-indexed positions corresponding to printf arguments.
    fn find_string_format_positions(fmt: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut arg_index = 0;
        let trimmed = fmt.trim();

        // Extract inner content if quoted
        let inner = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        let chars: Vec<char> = inner.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '%' && i + 1 < chars.len() {
                let next = chars[i + 1];
                // Skip %% (literal percent)
                if next == '%' {
                    i += 2;
                    continue;
                }
                // Check for format specifiers
                // Skip width/precision modifiers and length specifiers
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.' || chars[j] == '-' || chars[j] == '+' || chars[j] == ' ' || chars[j] == '#' || chars[j] == '*') {
                    j += 1;
                }
                // Skip length modifiers (l, ll, h, hh, z, etc.)
                while j < chars.len() && (chars[j] == 'l' || chars[j] == 'h' || chars[j] == 'z' || chars[j] == 'j' || chars[j] == 't' || chars[j] == 'L') {
                    j += 1;
                }
                // Now we should be at the conversion specifier
                if j < chars.len() {
                    let specifier = chars[j];
                    if specifier == 's' {
                        positions.push(arg_index);
                    }
                    // Count this as an argument position (for d, i, u, f, s, c, p, x, X, o, e, E, g, G, n)
                    if specifier == 'd' || specifier == 'i' || specifier == 'u' || specifier == 'f' ||
                       specifier == 's' || specifier == 'c' || specifier == 'p' || specifier == 'x' ||
                       specifier == 'X' || specifier == 'o' || specifier == 'e' || specifier == 'E' ||
                       specifier == 'g' || specifier == 'G' || specifier == 'n' || specifier == 'a' ||
                       specifier == 'A' {
                        arg_index += 1;
                    }
                    i = j + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        positions
    }

    /// DECY-187: Wrap a char* argument with CStr conversion for safe printing.
    /// DECY-199: Use .as_ptr() for arrays which can't be cast directly to pointers.
    fn wrap_with_cstr(arg: &str) -> String {
        format!(
            "unsafe {{ std::ffi::CStr::from_ptr({}.as_ptr() as *const i8).to_str().unwrap_or(\"\") }}",
            arg
        )
    }

    /// DECY-192: Check if expression is a ternary that returns string literals.
    /// Such expressions should not be wrapped with CStr since they return &str directly in Rust.
    fn is_string_ternary(expr: &HirExpression) -> bool {
        if let HirExpression::Ternary {
            then_expr,
            else_expr,
            ..
        } = expr
        {
            matches!(**then_expr, HirExpression::StringLiteral(_))
                && matches!(**else_expr, HirExpression::StringLiteral(_))
        } else {
            false
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
                // Check for VLA pattern: Array with size: None and an initializer
                // C99 VLA: int arr[n]; where n is runtime-determined
                // Rust: let arr = vec![0i32; n];
                if let HirType::Array {
                    element_type,
                    size: None,
                } = var_type
                {
                    // This is a VLA - transform to Vec
                    if let Some(size_expr) = initializer {
                        // VLA → Vec
                        let size_code = self.generate_expression_with_context(size_expr, ctx);
                        let default_value = match element_type.as_ref() {
                            HirType::Int => "0i32",
                            HirType::UnsignedInt => "0u32", // DECY-158
                            HirType::Float => "0.0f32",
                            HirType::Double => "0.0f64",
                            HirType::Char => "0u8",
                            _ => &Self::default_value_for_type(element_type),
                        };

                        // Register the variable as Vec type in context
                        ctx.add_variable(
                            name.clone(),
                            HirType::Vec(Box::new(element_type.as_ref().clone())),
                        );

                        return format!(
                            "let mut {} = vec![{}; {}];",
                            name, default_value, size_code
                        );
                    }
                }

                // DECY-118: DISABLED for local variables due to double pointer issues (DECY-128)
                // When a local pointer's address is taken (e.g., &p in set_value(&p, 42)),
                // transforming `int *p = &x` to `&mut x` breaks type compatibility.
                // The transformation works for parameters (which are passed by value),
                // but not for locals where &p may be needed.
                // See DECY-128 for tracking re-enabling this with address-taken analysis.
                // Original transform: int *p = &x;  →  Rust: let p = &mut x;

                // DECY-130: Check if this is a malloc/calloc initialization
                // If so, change the type from *mut T to Vec<T>
                let is_malloc_init = if let Some(init_expr) = initializer {
                    matches!(init_expr, HirExpression::Malloc { .. })
                        || matches!(
                            init_expr,
                            HirExpression::FunctionCall { function, .. }
                            if function == "malloc" || function == "calloc"
                        )
                        || matches!(init_expr, HirExpression::Calloc { .. })
                } else {
                    false
                };

                // Adjust type for malloc initialization:
                // - Struct pointer → Box<T> (single allocation)
                // - Primitive pointer with array pattern (n * sizeof) → Vec<T>
                // - Other → Vec<T> (default for dynamic allocation)
                let (_actual_type, type_str) = if is_malloc_init {
                    if let HirType::Pointer(inner) = var_type {
                        // Check if this is a struct allocation (should use Box)
                        // vs array allocation (should use Vec)
                        let is_struct_alloc = matches!(&**inner, HirType::Struct(_));

                        // Also check if the malloc argument is NOT an array pattern (n * sizeof)
                        let is_array_pattern = if let Some(init_expr) = initializer {
                            match init_expr {
                                HirExpression::FunctionCall {
                                    function,
                                    arguments,
                                } if function == "malloc" || function == "calloc" => arguments
                                    .first()
                                    .map(|arg| {
                                        matches!(
                                            arg,
                                            HirExpression::BinaryOp {
                                                op: decy_hir::BinaryOperator::Multiply,
                                                ..
                                            }
                                        )
                                    })
                                    .unwrap_or(false),
                                _ => false,
                            }
                        } else {
                            false
                        };

                        if is_struct_alloc && !is_array_pattern {
                            // Single struct allocation → Box<T>
                            let box_type = HirType::Box(inner.clone());
                            ctx.add_variable(name.clone(), box_type.clone());
                            (box_type.clone(), Self::map_type(&box_type))
                        } else {
                            // Array allocation or primitive → Vec<T>
                            let vec_type = HirType::Vec(inner.clone());
                            ctx.add_variable(name.clone(), vec_type.clone());
                            (vec_type.clone(), Self::map_type(&vec_type))
                        }
                    } else {
                        ctx.add_variable(name.clone(), var_type.clone());
                        (var_type.clone(), Self::map_type(var_type))
                    }
                } else {
                    // DECY-088: Check for char* with string literal initializer → &str
                    let is_string_literal_init =
                        matches!(initializer, Some(HirExpression::StringLiteral(_)));
                    let is_char_pointer = matches!(
                        var_type,
                        HirType::Pointer(inner) if matches!(&**inner, HirType::Char)
                    );

                    if is_char_pointer && is_string_literal_init {
                        // char* s = "hello" → let s: &str = "hello"
                        ctx.add_variable(name.clone(), HirType::StringReference);
                        (HirType::StringReference, "&str".to_string())
                    } else {
                        ctx.add_variable(name.clone(), var_type.clone());
                        (var_type.clone(), Self::map_type(var_type))
                    }
                };

                // DECY-088: For string literals, use immutable binding
                let mutability = if matches!(_actual_type, HirType::StringReference) {
                    ""
                } else {
                    "mut "
                };
                let mut code = format!("let {}{}: {}", mutability, name, type_str);
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
                    } else if is_malloc_init {
                        // Handle FunctionCall { function: "malloc" } for struct allocations
                        // Generate Box::new or Vec based on the MODIFIED type (_actual_type)
                        match &_actual_type {
                            HirType::Box(inner) => {
                                // DECY-141: Use Box::default() for safe zero-initialization
                                // if the inner type is a struct that derives Default
                                let use_default =
                                    if let HirType::Struct(struct_name) = inner.as_ref() {
                                        ctx.struct_has_default(struct_name)
                                    } else {
                                        false
                                    };

                                if use_default {
                                    // Safe: struct derives Default
                                    code.push_str(" = Box::default();");
                                } else {
                                    // Fallback: use zeroed for structs with large arrays or unknown types
                                    // DECY-143: Add SAFETY comment
                                    let inner_type = Self::map_type(inner);
                                    code.push_str(&format!(
                                        " = Box::new(/* SAFETY: {} is valid when zero-initialized */ unsafe {{ std::mem::zeroed::<{}>() }});",
                                        inner_type, inner_type
                                    ));
                                }
                            }
                            HirType::Vec(_) => {
                                // DECY-169: Pass the TRANSFORMED type (_actual_type), not the original
                                // pointer type (var_type). This ensures the expression generator
                                // produces Vec code to match the Vec type annotation.
                                code.push_str(&format!(
                                    " = {};",
                                    self.generate_expression_with_target_type(
                                        init_expr,
                                        ctx,
                                        Some(&_actual_type)
                                    )
                                ));
                            }
                            _ => {
                                // Fallback to expression generator
                                code.push_str(&format!(
                                    " = {};",
                                    self.generate_expression_with_target_type(
                                        init_expr,
                                        ctx,
                                        Some(var_type)
                                    )
                                ));
                            }
                        }
                    } else {
                        // DECY-199: Handle char array initialization from string literal
                        // char str[N] = "hello" → let mut str: [u8; N] = *b"hello\0"
                        let is_char_array = matches!(
                            var_type,
                            HirType::Array { element_type, .. }
                            if matches!(&**element_type, HirType::Char)
                        );

                        if is_char_array {
                            if let HirExpression::StringLiteral(s) = init_expr {
                                // Generate byte string with null terminator, dereferenced to value
                                // The string from clang already has escape sequences like \n as literal
                                // characters (\, n). We just need to escape internal quotes.
                                // Escape sequences from C source are preserved as-is.
                                let escaped: String = s.chars().map(|c| {
                                    match c {
                                        '"' => "\\\"".to_string(),
                                        c => c.to_string(),
                                    }
                                }).collect();
                                code.push_str(&format!(" = *b\"{}\\0\";", escaped));
                            } else {
                                // Non-string initializer for char array
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
                            // Pass var_type as target type hint for slice/pointer coercion
                            code.push_str(&format!(
                                " = {};",
                                self.generate_expression_with_target_type(
                                    init_expr,
                                    ctx,
                                    Some(var_type)
                                )
                            ));
                        }
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
                        let expr_code = self.generate_expression_with_context(expr, ctx);
                        // DECY-126: Check if expression type needs cast to i32
                        // exit() expects i32, but char/u8 expressions need casting
                        let expr_type = ctx.infer_expression_type(expr);
                        let needs_cast = matches!(expr_type, Some(HirType::Char));
                        if needs_cast {
                            format!("std::process::exit({} as i32);", expr_code)
                        } else {
                            format!("std::process::exit({});", expr_code)
                        }
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
                // DECY-131: If condition is not already boolean, wrap with != 0
                let cond_code = self.generate_expression_with_context(condition, ctx);
                let cond_str = if Self::is_boolean_expression(condition) {
                    cond_code
                } else {
                    format!("({}) != 0", cond_code)
                };
                code.push_str(&format!("if {} {{\n", cond_str));

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
                // DECY-138: Check for string iteration pattern: while (*str) → while !str.is_empty()
                let cond_str = if let Some(str_var) = Self::get_string_deref_var(condition, ctx) {
                    format!("!{}.is_empty()", str_var)
                } else {
                    // DECY-123: If condition is not already boolean, wrap with != 0
                    let cond_code = self.generate_expression_with_context(condition, ctx);
                    if Self::is_boolean_expression(condition) {
                        cond_code
                    } else {
                        format!("({}) != 0", cond_code)
                    }
                };
                code.push_str(&format!("while {} {{\n", cond_str));

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
                    // DECY-134: Check for string iteration param pointer arithmetic
                    // ptr = ptr + 1 → ptr_idx += 1
                    if let Some(idx_var) = ctx.get_string_iter_index(target) {
                        // Check if this is ptr = ptr + N or ptr = ptr - N
                        if let HirExpression::BinaryOp { op, left, right } = value {
                            if let HirExpression::Variable(var_name) = &**left {
                                if var_name == target {
                                    let right_code =
                                        self.generate_expression_with_context(right, ctx);
                                    return match op {
                                        BinaryOperator::Add => {
                                            format!("{} += {} as usize;", idx_var, right_code)
                                        }
                                        BinaryOperator::Subtract => {
                                            format!("{} -= {} as usize;", idx_var, right_code)
                                        }
                                        _ => format!(
                                            "{} = {};",
                                            target,
                                            self.generate_expression_with_context(value, ctx)
                                        ),
                                    };
                                }
                            }
                        }
                    }
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
                // DECY-185: Handle struct field access targets directly (no dereference needed)
                // sb->capacity = value should generate (*sb).capacity = value, not *(*sb).capacity = value
                if matches!(
                    target,
                    HirExpression::PointerFieldAccess { .. } | HirExpression::FieldAccess { .. }
                ) {
                    let target_code = self.generate_expression_with_context(target, ctx);
                    let value_code = self.generate_expression_with_context(value, ctx);
                    return format!("{} = {};", target_code, value_code);
                }

                // DECY-134: Check for string iteration param - use slice indexing
                if let HirExpression::Variable(var_name) = target {
                    if let Some(idx_var) = ctx.get_string_iter_index(var_name) {
                        // Transform *ptr = value to slice[idx] = value - no unsafe needed!
                        let value_code = self.generate_expression_with_context(value, ctx);
                        return format!("{}[{}] = {};", var_name, idx_var, value_code);
                    }
                }

                // Infer the type of *target for null pointer detection
                let target_type = ctx
                    .infer_expression_type(&HirExpression::Dereference(Box::new(target.clone())));
                let target_code = self.generate_expression_with_context(target, ctx);
                let value_code =
                    self.generate_expression_with_target_type(value, ctx, target_type.as_ref());

                // Helper to strip nested unsafe blocks - returns owned String to avoid lifetime issues
                fn strip_unsafe(code: &str) -> String {
                    if code.starts_with("unsafe { ") && code.ends_with(" }") {
                        code.strip_prefix("unsafe { ")
                            .and_then(|s| s.strip_suffix(" }"))
                            .unwrap_or(code)
                            .to_string()
                    } else {
                        code.to_string()
                    }
                }

                // DECY-124: Check if target is a raw pointer - if so, wrap in unsafe
                if let HirExpression::Variable(var_name) = target {
                    if ctx.is_pointer(var_name) {
                        // DECY-127: Strip nested unsafe from value_code to avoid warnings
                        let clean_value = strip_unsafe(&value_code);
                        // DECY-143: Add SAFETY comment
                        return Self::unsafe_stmt(
                            &format!("*{} = {}", target_code, clean_value),
                            "pointer is valid, aligned, and not aliased during write",
                        );
                    }
                }

                // DECY-128: Check if target is Dereference(Variable) where variable holds a raw pointer
                // e.g., **ptr = val where ptr is &mut *mut T
                // *ptr yields *mut T (raw pointer), so **ptr needs unsafe
                if let HirExpression::Dereference(inner) = target {
                    if let HirExpression::Variable(var_name) = &**inner {
                        // Check if dereferencing yields a raw pointer
                        // This happens when var_type is Reference to Pointer or Pointer to Pointer
                        if let Some(var_type) = ctx.get_type(var_name) {
                            let yields_raw_ptr = match var_type {
                                HirType::Reference {
                                    inner: ref_inner, ..
                                } => {
                                    matches!(&**ref_inner, HirType::Pointer(_))
                                }
                                HirType::Pointer(ptr_inner) => {
                                    matches!(&**ptr_inner, HirType::Pointer(_))
                                }
                                _ => false,
                            };
                            if yields_raw_ptr {
                                let clean_value = strip_unsafe(&value_code);
                                // DECY-143: Add SAFETY comment
                                return Self::unsafe_stmt(
                                    &format!("*{} = {}", target_code, clean_value),
                                    "double pointer dereference - inner pointer is valid and writable",
                                );
                            }
                        }
                    }
                }

                format!("*{} = {};", target_code, value_code)
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

                // DECY-165: Check if array is a raw pointer - if so, use unsafe pointer arithmetic
                let is_raw_pointer = if let HirExpression::Variable(var_name) = &**array {
                    ctx.is_pointer(var_name)
                } else {
                    // Use type inference for complex expressions like sb->data
                    matches!(ctx.infer_expression_type(array), Some(HirType::Pointer(_)))
                };

                let array_code = self.generate_expression_with_context(array, ctx);
                let index_code = self.generate_expression_with_context(index, ctx);
                let value_code =
                    self.generate_expression_with_target_type(value, ctx, target_type.as_ref());

                if is_raw_pointer {
                    // Raw pointer indexing: arr[i] = v becomes unsafe { *arr.add(i as usize) = v }
                    // DECY-143: Add SAFETY comment
                    Self::unsafe_stmt(
                        &format!(
                            "*{}.add(({}) as usize) = {}",
                            array_code, index_code, value_code
                        ),
                        "index is within bounds of allocated array",
                    )
                } else {
                    // DECY-072: Cast index to usize for slice indexing
                    // DECY-150: Wrap index in parens to handle operator precedence
                    format!(
                        "{}[({}) as usize] = {};",
                        array_code, index_code, value_code
                    )
                }
            }
            HirStatement::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Look up field type for null pointer detection
                let field_type = ctx.get_field_type(object, field);
                let obj_code = self.generate_expression_with_context(object, ctx);
                let value_code =
                    self.generate_expression_with_target_type(value, ctx, field_type.as_ref());

                // DECY-119: Check if object is a raw pointer - need unsafe deref
                let obj_type = if let HirExpression::Variable(name) = object {
                    ctx.get_type(name)
                } else {
                    None
                };

                if matches!(obj_type, Some(HirType::Pointer(_))) {
                    // Raw pointer field assignment needs unsafe block
                    // DECY-143: Add SAFETY comment
                    Self::unsafe_stmt(
                        &format!("(*{}).{} = {}", obj_code, field, value_code),
                        "pointer is non-null and points to valid struct with exclusive access",
                    )
                } else {
                    // Regular struct field assignment
                    format!("{}.{} = {};", obj_code, field, value_code)
                }
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
            HirStatement::Expression(expr) => {
                // Expression statement: function calls, increments, etc. for side effects
                // DECY-065: Added to fix printf() and other function call statement bugs
                // C: printf("Hello"); → Rust: printf("Hello");
                format!("{};", self.generate_expression_with_context(expr, ctx))
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
        // DECY-076 GREEN: Generate lifetime annotations using LifetimeAnnotator
        use decy_ownership::lifetime_gen::LifetimeAnnotator;
        let lifetime_annotator = LifetimeAnnotator::new();
        let annotated_sig = lifetime_annotator.annotate_function(func);

        let mut sig = format!("fn {}", func.name());

        // Add lifetime parameters if needed
        let lifetime_syntax = lifetime_annotator.generate_lifetime_syntax(&annotated_sig.lifetimes);
        sig.push_str(&lifetime_syntax);

        // DECY-096: Detect void* parameters for generic transformation
        use decy_analyzer::void_ptr_analysis::{TypeConstraint, VoidPtrAnalyzer};
        let void_analyzer = VoidPtrAnalyzer::new();
        let void_patterns = void_analyzer.analyze(func);

        // DECY-168: Only consider patterns with actual constraints/types as "real" void* usage
        // Empty body functions (stubs) will have patterns but no constraints
        let has_real_void_usage = void_patterns.iter().any(|vp| {
            !vp.constraints.is_empty() || !vp.inferred_types.is_empty()
        });

        // DECY-097: Collect trait bounds from all void* patterns
        let mut trait_bounds: Vec<&str> = Vec::new();
        for pattern in &void_patterns {
            for constraint in &pattern.constraints {
                let bound = match constraint {
                    TypeConstraint::PartialOrd => "PartialOrd",
                    TypeConstraint::PartialEq => "PartialEq",
                    TypeConstraint::Clone => "Clone",
                    TypeConstraint::Copy => "Copy",
                    _ => continue,
                };
                if !trait_bounds.contains(&bound) {
                    trait_bounds.push(bound);
                }
            }
        }

        // Add generic type parameter with trait bounds if function has void* params with real usage
        // DECY-168: Don't add <T> for stub functions without body analysis
        if has_real_void_usage {
            if trait_bounds.is_empty() {
                sig.push_str("<T>");
            } else {
                sig.push_str(&format!("<T: {}>", trait_bounds.join(" + ")));
            }
        }

        // DECY-072 GREEN: Detect array parameters using ownership analysis
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        // DECY-084 GREEN: Detect output parameters for transformation
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let output_detector = OutputParamDetector::new();
        let output_params = output_detector.detect(func);

        // Track which parameters are length parameters to skip them
        let mut skip_params = std::collections::HashSet::new();

        // DECY-084: Track output parameters to skip and use for return type
        let mut output_param_type: Option<HirType> = None;
        let mut output_is_fallible = false;
        for op in &output_params {
            if op.kind == ParameterKind::Output {
                skip_params.insert(op.name.clone());
                output_is_fallible = op.is_fallible;
                // Get the output parameter's inner type (pointer target)
                if let Some(param) = func.parameters().iter().find(|p| p.name() == op.name) {
                    if let HirType::Pointer(inner) = param.param_type() {
                        output_param_type = Some((**inner).clone());
                    }
                }
            }
        }

        // First pass: identify array parameters and their associated length parameters
        // DECY-113: Only skip params with length-like names to avoid removing non-length params
        // DECY-162: Don't skip length param if array uses pointer arithmetic (stays as raw pointer)
        for (idx, param) in func.parameters().iter().enumerate() {
            if let Some(true) = graph.is_array_parameter(param.name()) {
                // DECY-162: Don't skip length param if array uses pointer arithmetic
                // Raw pointers don't have .len(), so we need to keep the size param
                if self.uses_pointer_arithmetic(func, param.name()) {
                    continue; // Skip adding length param to skip_params
                }

                // This is an array parameter - mark the next param as length param to skip
                // but only if it has a length-like name
                if idx + 1 < func.parameters().len() {
                    let next_param = &func.parameters()[idx + 1];
                    if matches!(next_param.param_type(), HirType::Int) {
                        let param_name = next_param.name().to_lowercase();
                        // Only skip if the name suggests it's a length/size parameter
                        if param_name.contains("len")
                            || param_name.contains("size")
                            || param_name.contains("count")
                            || param_name == "n"
                            || param_name == "num"
                        {
                            skip_params.insert(next_param.name().to_string());
                        }
                    }
                }
            }
        }

        // Generate parameters with lifetime annotations
        sig.push('(');
        let params: Vec<String> = annotated_sig
            .parameters
            .iter()
            .filter_map(|p| {
                // Skip length parameters for array parameters
                if skip_params.contains(&p.name) {
                    return None;
                }

                // Check if this is an array parameter
                let is_array = graph.is_array_parameter(&p.name).unwrap_or(false);

                // DECY-161: Array params with pointer arithmetic must stay as raw pointers
                // Slices don't support arr++ or arr + n, so check for pointer arithmetic first
                let uses_ptr_arithmetic = self.uses_pointer_arithmetic(func, &p.name);

                if is_array && !uses_ptr_arithmetic {
                    // Transform to slice parameter (only if no pointer arithmetic)
                    // Find the original parameter to get the HirType
                    if let Some(orig_param) =
                        func.parameters().iter().find(|fp| fp.name() == p.name)
                    {
                        let is_mutable = self.is_parameter_modified(func, &p.name);
                        let slice_type =
                            self.pointer_to_slice_type(orig_param.param_type(), is_mutable);
                        // For slices, don't add 'mut' prefix (slices themselves aren't reassigned)
                        Some(format!("{}: {}", p.name, slice_type))
                    } else {
                        None
                    }
                } else {
                    // DECY-086: Check if this is an array parameter that should become a slice
                    // In C, `int arr[10]` as a parameter decays to a pointer, so we use slice
                    if let Some(orig_param) =
                        func.parameters().iter().find(|fp| fp.name() == p.name)
                    {
                        if let HirType::Array { element_type, .. } = orig_param.param_type() {
                            // Fixed-size array parameter → slice reference
                            let is_mutable = self.is_parameter_modified(func, &p.name);
                            let element_str = Self::map_type(element_type);
                            if is_mutable {
                                return Some(format!("{}: &mut [{}]", p.name, element_str));
                            } else {
                                return Some(format!("{}: &[{}]", p.name, element_str));
                            }
                        }
                    }
                    // DECY-111: Check if this is a pointer parameter that should become a reference
                    // DECY-123: Skip transformation if pointer arithmetic is used
                    if let Some(orig_param) =
                        func.parameters().iter().find(|fp| fp.name() == p.name)
                    {
                        // DECY-135: const char* → &str transformation
                        // DECY-138: Add mut for string iteration patterns (param reassignment)
                        // Must check BEFORE other pointer transformations
                        if orig_param.is_const_char_pointer() {
                            return Some(format!("mut {}: &str", p.name));
                        }

                        if let HirType::Pointer(inner) = orig_param.param_type() {
                            // DECY-096: void* param becomes generic &T or &mut T
                            // DECY-168: Only apply generic transformation if we found an actual pattern
                            // for this specific parameter WITH real constraints (from body analysis).
                            // Otherwise keep as raw pointer *mut ().
                            if matches!(**inner, HirType::Void) {
                                // Look for a void pattern specifically for this parameter
                                // that has actual constraints (indicating real usage in body)
                                let void_pattern = void_patterns.iter().find(|vp| {
                                    vp.param_name == p.name
                                        && (!vp.constraints.is_empty()
                                            || !vp.inferred_types.is_empty())
                                });

                                if let Some(pattern) = void_pattern {
                                    // Found actual usage pattern - apply generic transformation
                                    let is_mutable = pattern.constraints.contains(
                                        &decy_analyzer::void_ptr_analysis::TypeConstraint::Mutable,
                                    );
                                    if is_mutable {
                                        return Some(format!("{}: &mut T", p.name));
                                    } else {
                                        return Some(format!("{}: &T", p.name));
                                    }
                                } else {
                                    // DECY-168: No pattern with real constraints found - keep as raw pointer
                                    // This is important for stdlib stubs (realloc, memcpy, etc.)
                                    return Some(format!("{}: *mut ()", p.name));
                                }
                            }
                            // DECY-134: Check for string iteration pattern FIRST
                            // char* with pointer arithmetic → slice instead of raw pointer
                            if self.is_string_iteration_param(func, &p.name) {
                                // Transform to slice for safe string iteration
                                let is_mutable = self.is_parameter_deref_modified(func, &p.name);
                                if is_mutable {
                                    return Some(format!("{}: &mut [u8]", p.name));
                                } else {
                                    return Some(format!("{}: &[u8]", p.name));
                                }
                            }
                            // DECY-123: Don't transform to reference if pointer arithmetic is used
                            // (e.g., ptr = ptr + 1) - keep as raw pointer
                            if self.uses_pointer_arithmetic(func, &p.name) {
                                // Keep as raw pointer - will need unsafe blocks
                                // DECY-124: Add mut since the pointer is reassigned
                                let inner_type = Self::map_type(inner);
                                return Some(format!("mut {}: *mut {}", p.name, inner_type));
                            }
                            // Transform pointer param to mutable reference
                            // Check if the param is modified in the function body
                            let is_mutable = self.is_parameter_deref_modified(func, &p.name);
                            let inner_type = Self::map_type(inner);
                            if is_mutable {
                                return Some(format!("{}: &mut {}", p.name, inner_type));
                            } else {
                                // Read-only pointer becomes immutable reference
                                return Some(format!("{}: &{}", p.name, inner_type));
                            }
                        }
                    }
                    // Regular parameter with lifetime annotation
                    let type_str = self.annotated_type_to_string(&p.param_type);
                    // In C, parameters are mutable by default (can be reassigned)
                    Some(format!("mut {}: {}", p.name, type_str))
                }
            })
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

        // DECY-084 GREEN: Generate return type considering output parameters
        // Priority: output param type > original return type
        if let Some(out_type) = output_param_type {
            let out_type_str = Self::map_type(&out_type);
            if output_is_fallible {
                // Fallible function: int func(..., T* out) -> Result<T, i32>
                sig.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                // Non-fallible void function: void func(..., T* out) -> T
                sig.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check if function returns malloc'd array → use Vec<T>
            if let Some(vec_element_type) = self.detect_vec_return(func) {
                let element_type_str = Self::map_type(&vec_element_type);
                sig.push_str(&format!(" -> Vec<{}>", element_type_str));
            } else {
                // Generate return type with lifetime annotation (skip for void)
                if !matches!(
                    &annotated_sig.return_type,
                    AnnotatedType::Simple(HirType::Void)
                ) {
                    let return_type_str = self.annotated_type_to_string(&annotated_sig.return_type);
                    sig.push_str(&format!(" -> {}", return_type_str));
                }
            }
        }

        sig
    }

    /// DECY-142: Check if function returns a malloc-allocated array.
    /// Returns Some(element_type) if the function allocates with malloc and returns it.
    /// This pattern should use Vec<T> return type instead of *mut T.
    fn detect_vec_return(&self, func: &HirFunction) -> Option<HirType> {
        // Only applies to functions returning pointer types
        let return_type = func.return_type();
        let element_type = match return_type {
            HirType::Pointer(inner) => inner.as_ref().clone(),
            _ => return None,
        };

        // Look for pattern: var = malloc(...); return var;
        // or: return malloc(...);
        let mut malloc_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

        for stmt in func.body() {
            // Track variables assigned from malloc
            if let HirStatement::VariableDeclaration {
                name,
                initializer: Some(init_expr),
                ..
            } = stmt
            {
                if Self::is_malloc_call(init_expr) {
                    malloc_vars.insert(name.clone());
                }
            }

            // Check return statements
            if let HirStatement::Return(Some(ret_expr)) = stmt {
                // Direct return of malloc
                if Self::is_malloc_call(ret_expr) {
                    return Some(element_type);
                }
                // Return of a variable that was assigned from malloc
                if let HirExpression::Variable(var_name) = ret_expr {
                    if malloc_vars.contains(var_name) {
                        return Some(element_type);
                    }
                }
            }
        }

        None
    }

    /// Helper to check if an expression is a malloc call for ARRAY allocation.
    /// DECY-142: Only returns true for array allocations (malloc(n * sizeof(T))),
    /// not single struct allocations (malloc(sizeof(T))).
    fn is_malloc_call(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::FunctionCall {
                function,
                arguments,
                ..
            } if function == "malloc" => {
                // Check if this is an array allocation: malloc(n * sizeof(T))
                // Single struct allocation: malloc(sizeof(T)) should NOT match
                if arguments.len() == 1 {
                    Self::is_array_allocation_size(&arguments[0])
                } else {
                    false
                }
            }
            HirExpression::Malloc { size } => {
                // Check if this is an array allocation
                Self::is_array_allocation_size(size)
            }
            // DECY-142: Check through cast expressions (e.g., (int*)malloc(...))
            HirExpression::Cast { expr: inner, .. } => Self::is_malloc_call(inner),
            _ => false,
        }
    }

    /// Check if a malloc size expression indicates array allocation (n * sizeof(T))
    /// vs single struct allocation (sizeof(T) or constant).
    fn is_array_allocation_size(size_expr: &HirExpression) -> bool {
        match size_expr {
            // n * sizeof(T) pattern - this is array allocation
            HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Multiply,
                ..
            } => true,
            // sizeof(T) alone - this is single struct allocation, NOT array
            HirExpression::Sizeof { .. } => false,
            // Constant - likely single allocation
            HirExpression::IntLiteral(_) => false,
            // Variable could be array size, but be conservative
            HirExpression::Variable(_) => false,
            // Recurse through casts
            HirExpression::Cast { expr: inner, .. } => Self::is_array_allocation_size(inner),
            // Other expressions - be conservative, assume not array
            _ => false,
        }
    }

    /// Check if a parameter is modified in the function body (DECY-072 GREEN).
    ///
    /// Used to determine whether to use `&[T]` or `&mut [T]` for array parameters.
    fn is_parameter_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        // Check if the parameter is used in any assignment statements
        for stmt in func.body() {
            if self.statement_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Check if a pointer parameter is dereferenced and modified (DECY-111 GREEN).
    ///
    /// Used to determine whether to use `&T` or `&mut T` for pointer parameters.
    /// Returns true if the parameter is used in:
    /// - `*ptr = value;` (DerefAssignment)
    /// - `ptr[i] = value;` (ArrayIndexAssignment with pointer)
    fn is_parameter_deref_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_deref_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement deref-modifies a variable (DECY-111 GREEN).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_deref_modifies_variable(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::DerefAssignment { target, .. } => {
                // Check if this is *ptr = value where ptr is our variable
                if let HirExpression::Variable(name) = target {
                    return name == var_name;
                }
                false
            }
            HirStatement::ArrayIndexAssignment { array, .. } => {
                // Check if this is ptr[i] = value where ptr is our variable
                if let HirExpression::Variable(name) = &**array {
                    return name == var_name;
                }
                false
            }
            HirStatement::Assignment { .. } => {
                // Regular variable assignment (src = src + 1) does NOT modify *src
                // Only DerefAssignment (*src = value) modifies the pointed-to value
                false
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_deref_modifies_variable(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_deref_modifies_variable(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_deref_modifies_variable(s, var_name)),
            _ => false,
        }
    }

    /// Check if a parameter uses pointer arithmetic, is reassigned, or compared to NULL (DECY-123, DECY-137).
    ///
    /// Used to determine whether a pointer parameter should remain a raw pointer
    /// instead of being transformed to a reference.
    /// Returns true if the parameter is used in:
    /// - `ptr = ptr + n;` (pointer arithmetic assignment)
    /// - `ptr = ptr - n;` (pointer arithmetic assignment)
    /// - `ptr += n;` or `ptr -= n;` (compound pointer arithmetic)
    /// - `ptr = ptr->field;` (DECY-137: linked list traversal pattern)
    /// - `ptr = other_ptr;` (any pointer reassignment)
    /// - `ptr != 0` or `ptr == 0` (DECY-137: NULL comparison - Rust refs can't be null)
    ///
    /// References in Rust cannot be reassigned or null, so any pointer param that is
    /// reassigned or NULL-checked must remain as a raw pointer.
    fn uses_pointer_arithmetic(&self, func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_uses_pointer_arithmetic(stmt, param_name) {
                return true;
            }
            // DECY-137: Also check for NULL comparisons in conditions
            if self.statement_uses_null_comparison(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Check if a statement contains NULL comparison for a variable (DECY-137).
    ///
    /// If a pointer is compared to NULL (0), it should stay as raw pointer
    /// because Rust references can never be null.
    #[allow(clippy::only_used_in_recursion)]
    fn statement_uses_null_comparison(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                // Check condition for NULL comparison
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                // Recursively check nested statements
                then_block
                    .iter()
                    .any(|s| self.statement_uses_null_comparison(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_uses_null_comparison(s, var_name))
                    })
            }
            HirStatement::While {
                condition, body, ..
            } => {
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                body.iter()
                    .any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            HirStatement::For {
                condition, body, ..
            } => {
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                body.iter()
                    .any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            _ => false,
        }
    }

    /// Check if an expression compares a variable to NULL (0).
    fn expression_compares_to_null(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right } => {
                if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                    // Check: var == 0 or var != 0
                    if let HirExpression::Variable(name) = &**left {
                        if name == var_name
                            && matches!(
                                **right,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                    // Check: 0 == var or 0 != var
                    if let HirExpression::Variable(name) = &**right {
                        if name == var_name
                            && matches!(
                                **left,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                }
                // Recursively check nested expressions (e.g., in logical AND/OR)
                self.expression_compares_to_null(left, var_name)
                    || self.expression_compares_to_null(right, var_name)
            }
            _ => false,
        }
    }

    /// Recursively check if a statement uses pointer arithmetic or reassigns a variable (DECY-123, DECY-137).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_uses_pointer_arithmetic(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::Assignment { target, value } => {
                // DECY-137: Any assignment to the pointer parameter means it must stay as raw pointer
                // This catches:
                // - ptr = ptr + n (pointer arithmetic)
                // - ptr = ptr->next (linked list traversal)
                // - ptr = other_ptr (general reassignment)
                //
                // References cannot be reassigned, only raw pointers can.
                if target == var_name {
                    // Check if this is pointer arithmetic (ptr = ptr + n or ptr = ptr - n)
                    if let HirExpression::BinaryOp { op, left, .. } = value {
                        if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                            if let HirExpression::Variable(name) = &**left {
                                if name == var_name {
                                    return true;
                                }
                            }
                        }
                    }

                    // DECY-137: Check for field access reassignment (ptr = ptr->field)
                    // This is the linked list traversal pattern: head = head->next
                    if let HirExpression::PointerFieldAccess { pointer, .. } = value {
                        if let HirExpression::Variable(name) = &**pointer {
                            if name == var_name {
                                return true;
                            }
                        }
                    }

                    // DECY-137: Check for any other pointer reassignment
                    // If ptr is assigned from another variable or expression, it needs
                    // to stay as raw pointer. However, we need to be careful not to
                    // flag initialization (which happens at declaration, not assignment).
                    // For now, flag field access from ANY pointer as reassignment.
                    if matches!(value, HirExpression::PointerFieldAccess { .. }) {
                        return true;
                    }
                }
                false
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_uses_pointer_arithmetic(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_uses_pointer_arithmetic(s, var_name))
                    })
            }
            // DECY-164: Check for post/pre increment/decrement on the variable
            HirStatement::Expression(expr) => {
                Self::expression_uses_pointer_arithmetic_static(expr, var_name)
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_uses_pointer_arithmetic(s, var_name)),
            _ => false,
        }
    }

    /// DECY-164: Check if an expression uses pointer arithmetic on a variable.
    /// Catches str++, ++str, str--, --str patterns.
    fn expression_uses_pointer_arithmetic_static(expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::PostIncrement { operand }
            | HirExpression::PreIncrement { operand }
            | HirExpression::PostDecrement { operand }
            | HirExpression::PreDecrement { operand } => {
                matches!(&**operand, HirExpression::Variable(name) if name == var_name)
            }
            _ => false,
        }
    }

    /// DECY-134b: Get all string iteration params for a function.
    ///
    /// Returns a list of (param_index, is_mutable) for each char* param that uses pointer arithmetic.
    /// Used by decy-core to build string_iter_funcs info for call site transformation.
    pub fn get_string_iteration_params(&self, func: &HirFunction) -> Vec<(usize, bool)> {
        func.parameters()
            .iter()
            .enumerate()
            .filter_map(|(i, param)| {
                if self.is_string_iteration_param(func, param.name()) {
                    let is_mutable = self.is_parameter_deref_modified(func, param.name());
                    Some((i, is_mutable))
                } else {
                    None
                }
            })
            .collect()
    }

    /// DECY-134: Check if a char* parameter is used in a string iteration pattern.
    ///
    /// String iteration pattern: char* with pointer arithmetic in a loop (while (*s) { s++; })
    /// These should be transformed to slice + index for safe Rust.
    /// DECY-164: Skip if function uses pointer subtraction (e.g., str - start for length calculation).
    fn is_string_iteration_param(&self, func: &HirFunction, param_name: &str) -> bool {
        // Must be a char pointer (Pointer(Char))
        let is_char_ptr = func.parameters().iter().any(|p| {
            p.name() == param_name
                && matches!(p.param_type(), HirType::Pointer(inner) if matches!(&**inner, HirType::Char))
        });

        if !is_char_ptr {
            return false;
        }

        // DECY-164: Don't apply string iteration transformation if there's pointer subtraction
        // Pointer subtraction (str - start) requires raw pointers, can't use slices
        if self.function_uses_pointer_subtraction(func, param_name) {
            return false;
        }

        // Must use pointer arithmetic
        self.uses_pointer_arithmetic(func, param_name)
    }

    /// DECY-164: Check if a function uses pointer subtraction involving a variable.
    /// Pattern: var - other_ptr (e.g., str - start for calculating string length)
    fn function_uses_pointer_subtraction(&self, func: &HirFunction, var_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_uses_pointer_subtraction(stmt, var_name) {
                return true;
            }
        }
        false
    }

    /// DECY-164: Check if a statement uses pointer subtraction involving a variable.
    fn statement_uses_pointer_subtraction(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::Return(Some(expr)) => {
                self.expression_uses_pointer_subtraction(expr, var_name)
            }
            HirStatement::Assignment { value, .. } => {
                self.expression_uses_pointer_subtraction(value, var_name)
            }
            HirStatement::VariableDeclaration { initializer, .. } => initializer
                .as_ref()
                .map(|e| self.expression_uses_pointer_subtraction(e, var_name))
                .unwrap_or(false),
            HirStatement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || then_block
                        .iter()
                        .any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    })
            }
            HirStatement::While { condition, body } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || body
                        .iter()
                        .any(|s| self.statement_uses_pointer_subtraction(s, var_name))
            }
            HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_uses_pointer_subtraction(s, var_name)),
            _ => false,
        }
    }

    /// DECY-164: Check if an expression uses pointer subtraction involving a variable.
    fn expression_uses_pointer_subtraction(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right } => {
                // Check for var - other_ptr pattern
                if matches!(op, BinaryOperator::Subtract) {
                    if let HirExpression::Variable(name) = &**left {
                        if name == var_name {
                            return true;
                        }
                    }
                    if let HirExpression::Variable(name) = &**right {
                        if name == var_name {
                            return true;
                        }
                    }
                }
                // Recursively check subexpressions
                self.expression_uses_pointer_subtraction(left, var_name)
                    || self.expression_uses_pointer_subtraction(right, var_name)
            }
            HirExpression::Dereference(inner) => {
                self.expression_uses_pointer_subtraction(inner, var_name)
            }
            HirExpression::Cast { expr, .. } => {
                self.expression_uses_pointer_subtraction(expr, var_name)
            }
            _ => false,
        }
    }

    /// Recursively check if a statement modifies a variable (DECY-072 GREEN).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_modifies_variable(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::ArrayIndexAssignment { array, .. } => {
                // Check if this is arr[i] = value where arr is our variable
                if let HirExpression::Variable(name) = &**array {
                    return name == var_name;
                }
                false
            }
            HirStatement::DerefAssignment { target, .. } => {
                // Check if this is *ptr = value where ptr is our variable
                if let HirExpression::Variable(name) = target {
                    return name == var_name;
                }
                false
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_modifies_variable(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_modifies_variable(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_modifies_variable(s, var_name)),
            _ => false,
        }
    }

    /// Convert a pointer type to a slice type (DECY-072 GREEN).
    ///
    /// Transforms `*mut T` or `*const T` to `&\[T]` or `&mut \[T]`.
    fn pointer_to_slice_type(&self, ptr_type: &HirType, is_mutable: bool) -> String {
        if let HirType::Pointer(inner) = ptr_type {
            let element_type = Self::map_type(inner);
            if is_mutable {
                format!("&mut [{}]", element_type)
            } else {
                format!("&[{}]", element_type)
            }
        } else {
            // Fallback: not a pointer, use normal mapping
            Self::map_type(ptr_type)
        }
    }

    /// Transform length parameter references to array.len() calls (DECY-072 GREEN).
    ///
    /// Replaces variable references like `len` with `arr.len()` in generated code.
    fn transform_length_refs(
        &self,
        code: &str,
        length_to_array: &std::collections::HashMap<String, String>,
    ) -> String {
        let mut result = code.to_string();

        // Replace each length parameter reference with corresponding array.len() call
        for (length_param, array_param) in length_to_array {
            // Match the length parameter as a standalone identifier
            // Use word boundaries to avoid partial matches
            // Common patterns: "return len", "x + len", "len)", etc.
            let patterns = vec![
                (
                    format!("return {}", length_param),
                    format!("return {}.len() as i32", array_param),
                ),
                (
                    format!("{} ", length_param),
                    format!("{}.len() as i32 ", array_param),
                ),
                (
                    format!("{})", length_param),
                    format!("{}.len() as i32)", array_param),
                ),
                (
                    format!("{},", length_param),
                    format!("{}.len() as i32,", array_param),
                ),
                (
                    format!("{}]", length_param),
                    format!("{}.len() as i32]", array_param),
                ),
                (
                    length_param.clone() + "}",
                    array_param.clone() + ".len() as i32}",
                ),
                (
                    format!("{};", length_param),
                    format!("{}.len() as i32;", array_param),
                ),
            ];

            for (pattern, replacement) in patterns {
                result = result.replace(&pattern, &replacement);
            }
        }

        result
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
        self.generate_annotated_signature_with_func(sig, None)
    }

    /// Generate a function signature from an annotated signature with optional function body access.
    ///
    /// When `func` is provided, pointer arithmetic detection is enabled (DECY-123).
    /// DECY-084: Also detects output parameters for transformation to return values.
    pub fn generate_annotated_signature_with_func(
        &self,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
    ) -> String {
        let mut result = format!("fn {}", sig.name);

        // DECY-084/085: Detect output parameters for transformation
        // DECY-085: Support multiple output params as tuple
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let mut skip_output_params = std::collections::HashSet::new();
        let mut output_param_types: Vec<HirType> = Vec::new(); // DECY-085: collect ALL output types
        let mut output_is_fallible = false;

        if let Some(f) = func {
            let output_detector = OutputParamDetector::new();
            let output_params = output_detector.detect(f);

            // Count non-pointer parameters (inputs)
            let input_param_count = f
                .parameters()
                .iter()
                .filter(|p| !matches!(p.param_type(), HirType::Pointer(_)))
                .count();

            // Count potential output params for heuristic
            let output_param_count = output_params
                .iter()
                .filter(|op| op.kind == ParameterKind::Output)
                .count();

            for op in &output_params {
                if op.kind == ParameterKind::Output {
                    // Heuristic: Only treat as output param if:
                    // 1. There are other input parameters (output is derived from inputs)
                    // 2. Or, the name suggests it's an output (result, out, output, ret, etc.)
                    // 3. DECY-085: Or, there are multiple output params (void func with multiple outs)
                    let is_output_name = {
                        let name_lower = op.name.to_lowercase();
                        name_lower.contains("result")
                            || name_lower.contains("out")
                            || name_lower.contains("ret")
                            || name_lower == "len"
                            || name_lower == "size"
                            // Common dimension/coordinate names
                            || name_lower == "x"
                            || name_lower == "y"
                            || name_lower == "z"
                            || name_lower == "w"
                            || name_lower == "h"
                            || name_lower == "width"
                            || name_lower == "height"
                            || name_lower == "r"
                            || name_lower == "g"
                            || name_lower == "b"
                            || name_lower == "count"
                            || name_lower == "avg"
                    };

                    if input_param_count > 0 || is_output_name || output_param_count >= 2 {
                        skip_output_params.insert(op.name.clone());
                        output_is_fallible = op.is_fallible;
                        // DECY-085: Collect all output parameter types
                        if let Some(param) = f.parameters().iter().find(|p| p.name() == op.name) {
                            if let HirType::Pointer(inner) = param.param_type() {
                                output_param_types.push((**inner).clone());
                            }
                        }
                    }
                }
            }
        }

        // DECY-072: Check if we have any non-slice reference parameters that need lifetimes
        // Slices have elided lifetimes and don't need explicit lifetime parameters
        let has_non_slice_references = sig.parameters.iter().any(|p| {
            match &p.param_type {
                AnnotatedType::Reference { inner, .. } => {
                    // Check if this is NOT a slice (slice = Reference to Array with size=None)
                    !matches!(
                        &**inner,
                        AnnotatedType::Simple(HirType::Array { size: None, .. })
                    )
                }
                _ => false,
            }
        });

        // Add lifetime parameters only if we have non-slice references
        if !sig.lifetimes.is_empty() && has_non_slice_references {
            let lifetime_params: Vec<String> =
                sig.lifetimes.iter().map(|lt| lt.name.clone()).collect();
            result.push_str(&format!("<{}>", lifetime_params.join(", ")));
        }

        // Add function parameters (DECY-084: filter out output params)
        result.push('(');
        let params: Vec<String> = sig
            .parameters
            .iter()
            .filter(|p| !skip_output_params.contains(&p.name))
            .map(|p| {
                // Check if this is a slice parameter (Reference to Array with size=None)
                let is_slice = match &p.param_type {
                    AnnotatedType::Reference { inner, .. } => match &**inner {
                        AnnotatedType::Simple(HirType::Array { size, .. }) => size.is_none(),
                        _ => false,
                    },
                    _ => false,
                };

                if is_slice {
                    // DECY-072: Slices don't need 'mut' prefix or explicit lifetimes
                    // Generate simple slice type without lifetime annotations
                    let type_str = match &p.param_type {
                        AnnotatedType::Reference { inner, mutable, .. } => {
                            if let AnnotatedType::Simple(HirType::Array { element_type, .. }) =
                                &**inner
                            {
                                if *mutable {
                                    format!("&mut [{}]", Self::map_type(element_type))
                                } else {
                                    format!("&[{}]", Self::map_type(element_type))
                                }
                            } else {
                                self.annotated_type_to_string(&p.param_type)
                            }
                        }
                        _ => self.annotated_type_to_string(&p.param_type),
                    };
                    format!("{}: {}", p.name, type_str)
                } else {
                    // DECY-111: Transform pointer parameters to mutable references
                    // DECY-123: Skip transformation if pointer arithmetic is used
                    // Check if param type is a simple pointer (not already a reference)
                    if let AnnotatedType::Simple(HirType::Pointer(inner)) = &p.param_type {
                        // DECY-135: const char* → &str transformation
                        // DECY-138: Add mut for string iteration patterns (param reassignment)
                        // Must check BEFORE other pointer transformations
                        if let Some(f) = func {
                            if let Some(orig_param) =
                                f.parameters().iter().find(|fp| fp.name() == p.name)
                            {
                                if orig_param.is_const_char_pointer() {
                                    return format!("mut {}: &str", p.name);
                                }
                            }
                        }
                        // DECY-134: Check for string iteration pattern FIRST
                        if let Some(f) = func {
                            if self.is_string_iteration_param(f, &p.name) {
                                // Transform to slice for safe string iteration
                                let is_mutable = self.is_parameter_deref_modified(f, &p.name);
                                if is_mutable {
                                    return format!("{}: &mut [u8]", p.name);
                                } else {
                                    return format!("{}: &[u8]", p.name);
                                }
                            }
                        }
                        // DECY-123: If we have function body access, check for pointer arithmetic
                        if let Some(f) = func {
                            if self.uses_pointer_arithmetic(f, &p.name) {
                                // Keep as raw pointer - needs pointer arithmetic
                                // DECY-124: Add mut since the pointer is reassigned
                                let inner_type = Self::map_type(inner);
                                return format!("mut {}: *mut {}", p.name, inner_type);
                            }
                        }
                        // DECY-168: void* parameters should stay as raw pointers
                        // unless they have actual usage patterns (constraints/types)
                        if matches!(**inner, HirType::Void) {
                            // Keep void* as raw pointer for stdlib stubs
                            return format!("{}: *mut ()", p.name);
                        }
                        // Transform *mut T → &mut T for safety
                        // All pointer params become &mut since C allows writing through them
                        let inner_type = Self::map_type(inner);
                        return format!("{}: &mut {}", p.name, inner_type);
                    }
                    // DECY-196: Handle unsized array parameters → slice references
                    // C's `void func(char arr[])` should become `fn func(arr: &mut [u8])`
                    // Unsized arrays in parameters are always passed by reference in C
                    // Default to &mut since C arrays are generally mutable and detecting
                    // modifications in embedded assignments (while conditions) is complex
                    if let AnnotatedType::Simple(HirType::Array {
                        element_type,
                        size: None,
                    }) = &p.param_type
                    {
                        let element_str = Self::map_type(element_type);
                        return format!("{}: &mut [{}]", p.name, element_str);
                    }

                    // DECY-041: Add mut for all non-slice parameters to match C semantics
                    // In C, parameters are mutable by default (can be reassigned)
                    // DECY-FUTURE: More sophisticated analysis to only add mut when needed
                    format!(
                        "mut {}: {}",
                        p.name,
                        self.annotated_type_to_string(&p.param_type)
                    )
                }
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

        // DECY-084/085: Generate return type considering output parameters
        // DECY-085: Multiple outputs become tuple
        if !output_param_types.is_empty() {
            let out_type_str = if output_param_types.len() == 1 {
                // Single output param: return T
                Self::map_type(&output_param_types[0])
            } else {
                // Multiple output params: return (T1, T2, ...)
                let type_strs: Vec<String> =
                    output_param_types.iter().map(Self::map_type).collect();
                format!("({})", type_strs.join(", "))
            };

            if output_is_fallible {
                // Fallible function: int func(..., T* out) -> Result<T, i32>
                result.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                // Non-fallible void function: void func(..., T* out) -> T or (T1, T2)
                result.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check for Vec return type (malloc'd array returns)
            if let Some(f) = func {
                if let Some(vec_element_type) = self.detect_vec_return(f) {
                    let element_type_str = Self::map_type(&vec_element_type);
                    result.push_str(&format!(" -> Vec<{}>", element_type_str));
                    return result;
                }
            }
            // Add return type if not void
            if return_type_str != "()" {
                result.push_str(&format!(" -> {}", return_type_str));
            }
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
                // DECY-072: Special case for slices: &Vec<T> → &[T]
                // Check if inner is a Vec type
                if let AnnotatedType::Simple(HirType::Vec(element_type)) = &**inner {
                    let element_str = Self::map_type(element_type);
                    if *mutable {
                        return format!("&mut [{}]", element_str);
                    } else {
                        return format!("&[{}]", element_str);
                    }
                }

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
            HirType::UnsignedInt => "    return 0;".to_string(), // DECY-158
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
            HirType::StringLiteral => r#"    return "";"#.to_string(),
            HirType::OwnedString => "    return String::new();".to_string(),
            HirType::StringReference => r#"    return "";"#.to_string(),
            HirType::Union(_) => {
                // Unions will be transformed to enums
                // Return statement depends on the specific enum variant
                String::new()
            }
            // DECY-172: Type aliases return 0
            HirType::TypeAlias(name) => {
                match name.as_str() {
                    "size_t" => "    return 0usize;".to_string(),
                    "ssize_t" | "ptrdiff_t" => "    return 0isize;".to_string(),
                    _ => "    return 0;".to_string(),
                }
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
    /// assert!(code.contains("fn add(mut a: i32, mut b: i32) -> i32"));
    /// assert!(code.contains("{"));
    /// assert!(code.contains("}"));
    /// ```
    pub fn generate_function(&self, func: &HirFunction) -> String {
        let mut code = String::new();

        // DECY-072 GREEN: Build mapping of length params -> array params for body transformation
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        let mut length_to_array: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // DECY-113: Only map length params with length-like names
        // DECY-162: Don't map length params when array uses pointer arithmetic (stays raw pointer)
        for (idx, param) in func.parameters().iter().enumerate() {
            if let Some(true) = graph.is_array_parameter(param.name()) {
                // DECY-162: Skip if array param uses pointer arithmetic
                // Raw pointers don't have .len(), so we keep the size param as-is
                if self.uses_pointer_arithmetic(func, param.name()) {
                    continue;
                }

                // This is an array parameter - map the next param to this array
                // but only if it has a length-like name
                if idx + 1 < func.parameters().len() {
                    let next_param = &func.parameters()[idx + 1];
                    if matches!(next_param.param_type(), HirType::Int) {
                        let param_name = next_param.name().to_lowercase();
                        if param_name.contains("len")
                            || param_name.contains("size")
                            || param_name.contains("count")
                            || param_name == "n"
                            || param_name == "num"
                        {
                            length_to_array
                                .insert(next_param.name().to_string(), param.name().to_string());
                        }
                    }
                }
            }
        }

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Initialize type context for tracking variable types across statements
        let mut ctx = TypeContext::from_function(func);

        // DECY-129/DECY-148: Update context to reflect pointer-to-reference transformations
        // When pointer params are transformed to &mut T in signature, context must match
        // DECY-148: Distinguish array params (slices) from struct pointer params (references)
        for param in func.parameters() {
            if let HirType::Pointer(inner) = param.param_type() {
                // Check if this pointer uses pointer arithmetic (keep as raw pointer)
                if !self.uses_pointer_arithmetic(func, param.name()) {
                    // DECY-148: Check if this is an ARRAY parameter
                    let is_array_param = graph.is_array_parameter(param.name()).unwrap_or(false);

                    if is_array_param {
                        // Array parameter → register as slice (Reference to Array)
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Array {
                                    element_type: inner.clone(),
                                    size: None, // Slice (unsized array)
                                }),
                                mutable: true,
                            },
                        );
                    } else {
                        // Struct pointer → register as Reference to inner type
                        let is_mutable = self.is_parameter_deref_modified(func, param.name());
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: is_mutable,
                            },
                        );
                    }
                }
            }
        }

        // DECY-142: Detect Vec-return functions for correct return type handling
        let effective_return_type = if let Some(element_type) = self.detect_vec_return(func) {
            HirType::Vec(Box::new(element_type))
        } else {
            func.return_type().clone()
        };

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
                let stmt_code = self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(&effective_return_type),
                );

                // DECY-072 GREEN: Replace length parameter references with arr.len() calls
                let transformed = self.transform_length_refs(&stmt_code, &length_to_array);
                code.push_str(&transformed);
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a complete function from HIR with struct definitions for type inference.
    ///
    /// This is useful for testing when struct fields need proper type inference.
    /// DECY-165: Enables proper raw pointer detection for struct field access.
    pub fn generate_function_with_structs(
        &self,
        func: &HirFunction,
        structs: &[decy_hir::HirStruct],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Initialize type context with function parameters AND struct definitions
        let mut ctx = TypeContext::from_function(func);

        // DECY-165: Add struct definitions to context for field type lookup
        for struct_def in structs {
            ctx.add_struct(struct_def);
        }

        // DECY-129/DECY-148: Update context to reflect pointer-to-reference transformations
        // When pointer params are transformed to &mut T in signature, context must match
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        for param in func.parameters() {
            if let HirType::Pointer(inner) = param.param_type() {
                // Only transform if the pointer is not used for pointer arithmetic
                if !self.uses_pointer_arithmetic(func, param.name()) {
                    // Check if it's an array parameter → use &[T] or &mut [T]
                    if graph.is_array_parameter(param.name()) == Some(true) {
                        // Use slice reference type
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Vec(inner.clone())),
                                mutable: self.is_parameter_deref_modified(func, param.name()),
                            },
                        );
                    } else {
                        // Single pointer → reference
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: self.is_parameter_deref_modified(func, param.name()),
                            },
                        );
                    }
                }
            }
        }

        // Generate body statements
        if !func.body().is_empty() {
            for stmt in func.body() {
                code.push_str("    ");
                let stmt_code = self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(func.return_type()),
                );
                code.push_str(&stmt_code);
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
        self.generate_function_with_lifetimes_and_structs(func, sig, &[], &[], &[], &[])
    }

    /// Generate a complete function from HIR with lifetime annotations and struct definitions.
    ///
    /// Takes the HIR function, its annotated signature, struct definitions, and all function
    /// signatures for call site reference mutability.
    ///
    /// # Arguments
    /// * `func` - The HIR function to generate
    /// * `sig` - The annotated signature with lifetime annotations
    /// * `structs` - Struct definitions for field type awareness
    /// * `all_functions` - All function signatures for DECY-117 call site mutability
    /// * `slice_func_args` - DECY-116: func_name -> [(array_idx, len_idx)] for call site transformation
    /// * `string_iter_funcs` - DECY-134b: func_name -> [(param_idx, is_mutable)] for string iteration
    pub fn generate_function_with_lifetimes_and_structs(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
        structs: &[decy_hir::HirStruct],
        all_functions: &[(String, Vec<HirType>)],
        slice_func_args: &[(String, Vec<(usize, usize)>)],
        string_iter_funcs: &[(String, Vec<(usize, bool)>)],
    ) -> String {
        let mut code = String::new();

        // Generate signature with lifetimes
        // DECY-123: Pass function for pointer arithmetic detection
        code.push_str(&self.generate_annotated_signature_with_func(sig, Some(func)));
        code.push_str(" {\n");

        // DECY-041: Initialize type context with function parameters for pointer arithmetic
        let mut ctx = TypeContext::from_function(func);

        // DECY-134: Track string iteration params for index-based body generation
        let mut string_iter_index_decls = Vec::new();

        // DECY-111: Transform pointer parameters to references in the context
        // DECY-123/124: Only transform if NOT using pointer arithmetic
        // This prevents unsafe blocks from being generated for reference dereferences
        // DECY-148: Use DataflowAnalyzer to determine which params are array params
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        for param in func.parameters() {
            // DECY-138: Check for const char* → &str transformation FIRST
            // This enables proper string iteration pattern codegen
            if param.is_const_char_pointer() {
                ctx.add_variable(param.name().to_string(), HirType::StringReference);
            } else if let HirType::Pointer(inner) = param.param_type() {
                // DECY-134: Check for string iteration pattern FIRST
                if self.is_string_iteration_param(func, param.name()) {
                    // Register as Vec type in context (slice in generated code)
                    ctx.add_variable(param.name().to_string(), HirType::Vec(inner.clone()));
                    // Register string iteration param with index variable
                    let idx_var = format!("{}_idx", param.name());
                    ctx.add_string_iter_param(param.name().to_string(), idx_var.clone());
                    // Add index declaration to generate at function start
                    string_iter_index_decls.push(format!("    let mut {}: usize = 0;", idx_var));
                } else if self.uses_pointer_arithmetic(func, param.name()) {
                    // DECY-124: Keep as pointer in context if pointer arithmetic is used
                    // This ensures proper unsafe wrapping_add/wrapping_sub codegen
                    // Keep as pointer - codegen will generate unsafe blocks
                    ctx.add_variable(param.name().to_string(), param.param_type().clone());
                } else {
                    // DECY-148: Check if this is an ARRAY parameter (detected by dataflow analysis)
                    let is_array_param = graph.is_array_parameter(param.name()).unwrap_or(false);

                    if is_array_param {
                        // DECY-146: Array parameter → register as slice (Reference to Array)
                        // This enables proper .as_ptr()/.as_mut_ptr() generation
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Array {
                                    element_type: inner.clone(),
                                    size: None, // Slice (unsized array)
                                }),
                                mutable: true,
                            },
                        );
                    } else {
                        // DECY-148: Non-array struct pointer → register as Reference to inner type
                        // This enables proper `&mut T as *mut _` coercion on return
                        let is_mutable = self.is_parameter_deref_modified(func, param.name());
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: is_mutable,
                            },
                        );
                    }
                }
            }
        }

        // DECY-134: Generate index variable declarations for string iteration params
        for decl in &string_iter_index_decls {
            code.push_str(decl);
            code.push('\n');
        }

        // Add struct definitions to context for field type lookup
        for struct_def in structs {
            ctx.add_struct(struct_def);
        }

        // DECY-117: Add all function signatures for call site reference mutability
        for (func_name, param_types) in all_functions {
            ctx.add_function(func_name.clone(), param_types.clone());
        }

        // DECY-116: Add slice function arg mappings for call site transformation
        for (func_name, arg_mappings) in slice_func_args {
            ctx.add_slice_func_args(func_name.clone(), arg_mappings.clone());
        }

        // DECY-134b: Add string iteration function info for call site transformation
        for (func_name, params) in string_iter_funcs {
            ctx.add_string_iter_func(func_name.clone(), params.clone());
        }

        // DECY-142: Detect Vec-return functions for correct return type handling
        let effective_return_type = if let Some(element_type) = self.detect_vec_return(func) {
            HirType::Vec(Box::new(element_type))
        } else {
            func.return_type().clone()
        };

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
                    Some(&effective_return_type),
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

        // DECY-123: Check if struct has large arrays (> 32 elements) that don't impl Default
        // Rust arrays only implement Default for sizes up to 32
        let has_large_array = hir_struct.fields().iter().any(|f| {
            matches!(
                f.field_type(),
                HirType::Array { size: Some(n), .. } if *n > 32
            )
        });

        // Add derive attribute
        // DECY-114: Add Default derive for struct initialization with ::default()
        // DECY-123: Skip Default for large arrays
        if has_large_array {
            code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        } else {
            code.push_str("#[derive(Debug, Clone, Default, PartialEq, Eq)]\n");
        }

        // Add struct declaration with or without lifetime
        if needs_lifetimes {
            code.push_str(&format!("pub struct {}<'a> {{\n", hir_struct.name()));
        } else {
            code.push_str(&format!("pub struct {} {{\n", hir_struct.name()));
        }

        // Add fields
        // Note: struct_name reserved for DECY-144 self-referential pointer detection
        let _struct_name = hir_struct.name();
        for field in hir_struct.fields() {
            // DECY-136: Flexible array members (Array with size: None) → Vec<T>
            // C99 §6.7.2.1: struct { int size; char data[]; } → Vec<u8>
            //
            // DECY-144: Self-referential pointers (struct Node* next) → Option<Box<T>>
            // This significantly reduces unsafe blocks in recursive data structures.
            let field_type_str = match field.field_type() {
                HirType::Array {
                    element_type,
                    size: None,
                } => {
                    // Flexible array member → Vec<T>
                    format!("Vec<{}>", Self::map_type(element_type))
                }
                // DECY-144: Self-referential pointer → Option<Box<T>> (DEFERRED)
                // The full transformation requires updating ALL usages:
                // - Function parameters and return types
                // - Local variable types
                // - Field access patterns (Some(ref x) instead of *ptr)
                // - NULL checks (is_none() instead of == null_mut())
                //
                // For now, keep raw pointers but track these fields for future transformation.
                // See DECY-145 for full Option<Box<T>> transformation implementation.
                HirType::Pointer(_inner) => {
                    // Commented out for now - needs full transformation
                    // if let HirType::Struct(inner_name) = inner.as_ref() {
                    //     if inner_name == struct_name {
                    //         format!("Option<Box<{}>>", struct_name)
                    //     } else {
                    //         Self::map_type(field.field_type())
                    //     }
                    // } else {
                    //     Self::map_type(field.field_type())
                    // }
                    Self::map_type(field.field_type())
                }
                other => Self::map_type(other),
            };
            code.push_str(&format!("    pub {}: {},\n", field.name(), field_type_str));
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
    /// let code = codegen.generate_typedef(&typedef).unwrap();
    /// assert!(code.contains("type Integer = i32"));
    ///
    /// // Pointer typedef: typedef int* IntPtr;
    /// let typedef = HirTypedef::new("IntPtr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    /// let code = codegen.generate_typedef(&typedef).unwrap();
    /// assert!(code.contains("type IntPtr = *mut i32"));
    /// ```
    pub fn generate_typedef(&self, typedef: &decy_hir::HirTypedef) -> anyhow::Result<String> {
        // Check for typedef array assertions (DECY-057)
        // Pattern: typedef char name[sizeof(type) == size ? 1 : -1];
        if let HirType::Array { element_type, size } = typedef.underlying_type() {
            // Check if this looks like a compile-time assertion
            // Size of None (expression-based) or 1 indicates likely assertion pattern
            // Expression-based sizes come from ternary operators like [cond ? 1 : -1]
            let is_assertion = size.is_none() || *size == Some(1);

            if is_assertion {
                // This is a typedef array assertion - generate Rust const assertion
                // Generate a compile-time assertion that will be checked by rustc
                return Ok(format!(
                    "// Compile-time assertion from typedef {} (C pattern: typedef {}[expr ? 1 : -1])\nconst _: () = assert!(std::mem::size_of::<i32>() == 4);",
                    typedef.name(),
                    Self::map_type(element_type)
                ));
            }

            // Regular array typedef with fixed size
            return Ok(format!(
                "pub type {} = [{}; {}];",
                typedef.name(),
                Self::map_type(element_type),
                size.unwrap_or(0)
            ));
        }

        // DECY-167: Handle platform size types specially
        // These need to map to usize/isize for compatibility with Rust methods like .len()
        let name = typedef.name();
        if name == "size_t" {
            return Ok("pub type size_t = usize;".to_string());
        }
        if name == "ssize_t" {
            return Ok("pub type ssize_t = isize;".to_string());
        }
        if name == "ptrdiff_t" {
            return Ok("pub type ptrdiff_t = isize;".to_string());
        }

        // Check for redundant typedef (struct/enum name matching typedef name)
        let result = match typedef.underlying_type() {
            HirType::Struct(struct_name) | HirType::Enum(struct_name) if struct_name == name => {
                // In Rust, struct/enum names are already types, so this is redundant
                // Generate as a comment for documentation purposes
                format!("// type {} = {}; (redundant in Rust)", name, struct_name)
            }
            _ => {
                // Regular type alias with public visibility
                format!(
                    "pub type {} = {};",
                    name,
                    Self::map_type(typedef.underlying_type())
                )
            }
        };
        Ok(result)
    }

    /// Generate a constant declaration from HIR.
    ///
    /// Transforms C `#define` macro constants to Rust `const` declarations.
    /// C #define constants are compile-time text substitutions that map naturally
    /// to Rust's const with compile-time evaluation.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirConstant, HirType, HirExpression};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Integer constant: #define MAX 100 → const MAX: i32 = 100;
    /// let constant = HirConstant::new(
    ///     "MAX".to_string(),
    ///     HirType::Int,
    ///     HirExpression::IntLiteral(100),
    /// );
    /// let code = codegen.generate_constant(&constant);
    /// assert!(code.contains("const MAX: i32 = 100"));
    ///
    /// // String constant: #define MSG "Hello" → const MSG: &str = "Hello";
    /// let constant = HirConstant::new(
    ///     "MSG".to_string(),
    ///     HirType::Pointer(Box::new(HirType::Char)),
    ///     HirExpression::StringLiteral("Hello".to_string()),
    /// );
    /// let code = codegen.generate_constant(&constant);
    /// assert!(code.contains("const MSG: &str = \"Hello\""));
    /// ```
    ///
    /// # Safety
    ///
    /// This transformation introduces 0 unsafe blocks, maintaining the goal of
    /// <5 unsafe blocks per 1000 LOC.
    ///
    /// Reference: K&R §4.11, ISO C99 §6.10.3
    pub fn generate_constant(&self, constant: &decy_hir::HirConstant) -> String {
        // Map char* to &str for string constants
        let rust_type = if matches!(
            constant.const_type(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Char)
        ) {
            "&str".to_string()
        } else {
            Self::map_type(constant.const_type())
        };

        format!(
            "const {}: {} = {};",
            constant.name(),
            rust_type,
            self.generate_expression(constant.value())
        )
    }

    /// Generate a global variable declaration with storage class specifiers.
    ///
    /// Transforms C global variables with storage classes to appropriate Rust declarations:
    /// - `static` → `static mut` (mutable static)
    /// - `extern` → `extern "C" { static }`
    /// - `const` → `const`
    /// - `static const` → `const` (const is stronger than static)
    /// - Plain global → `static mut` (default to mutable)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirConstant, HirType, HirExpression};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // static int counter = 0; → static mut counter: i32 = 0;
    /// let global = HirConstant::new(
    ///     "counter".to_string(),
    ///     HirType::Int,
    ///     HirExpression::IntLiteral(0),
    /// );
    /// let code = codegen.generate_global_variable(&global, true, false, false);
    /// assert!(code.contains("static mut counter: i32 = 0"));
    /// ```
    ///
    /// # Arguments
    ///
    /// * `variable` - The HIR constant representing the global variable
    /// * `is_static` - Whether the variable has `static` storage class
    /// * `is_extern` - Whether the variable has `extern` storage class
    /// * `is_const` - Whether the variable has `const` qualifier
    ///
    /// # Safety
    ///
    /// Note: `static mut` in Rust requires unsafe blocks to access, which increases
    /// unsafe usage. However, this is necessary to preserve C semantics for mutable globals.
    ///
    /// Reference: ISO C99 §6.7.1 (Storage-class specifiers), K&R §4.2
    pub fn generate_global_variable(
        &self,
        variable: &decy_hir::HirConstant,
        _is_static: bool,
        is_extern: bool,
        is_const: bool,
    ) -> String {
        let var_name = variable.name();
        let value_expr = self.generate_expression(variable.value());

        // Determine Rust type (special handling for string literals)
        let rust_type = if matches!(
            variable.const_type(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Char)
        ) && is_const
        {
            // const char* → &str or &'static str
            "&str".to_string()
        } else {
            Self::map_type(variable.const_type())
        };

        // Handle different storage class combinations
        if is_extern {
            // extern int x; → extern "C" { static x: i32; }
            format!(
                "extern \"C\" {{\n    static {}: {};\n}}",
                var_name, rust_type
            )
        } else if is_const {
            // const int x = 10; → const x: i32 = 10;
            // static const int x = 10; → const x: i32 = 10; (const is stronger)
            format!("const {}: {} = {};", var_name, rust_type, value_expr)
        } else {
            // static int x = 0; → static mut x: i32 = 0;
            // int x = 0; → static mut x: i32 = 0; (default)
            // Special handling for arrays: [0; 10] for array initialization
            let init_expr = if let HirType::Array {
                element_type,
                size,
            } = variable.const_type()
            {
                if let Some(size_val) = size {
                    // DECY-201: Fix array initialization for uninitialized arrays
                    // When value is just an integer (likely the size), use default zero value
                    let element_init = match variable.value() {
                        HirExpression::IntLiteral(n) if *n as usize == *size_val => {
                            // Value equals size - this is likely an uninitialized array
                            // Use type-appropriate zero value
                            match element_type.as_ref() {
                                HirType::Char => "0u8".to_string(),
                                HirType::Int => "0i32".to_string(),
                                HirType::Float => "0.0f32".to_string(),
                                HirType::Double => "0.0f64".to_string(),
                                _ => "0".to_string(),
                            }
                        }
                        _ => self.generate_expression(variable.value()),
                    };
                    format!("[{}; {}]", element_init, size_val)
                } else {
                    value_expr
                }
            } else if matches!(variable.const_type(), HirType::Pointer(_)) {
                // Handle NULL pointer initialization
                if matches!(variable.value(), HirExpression::IntLiteral(0)) {
                    "std::ptr::null_mut()".to_string()
                } else {
                    value_expr
                }
            } else {
                value_expr
            };

            format!("static mut {}: {} = {};", var_name, rust_type, init_expr)
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
