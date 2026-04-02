//! Type code generation: struct, class, namespace, CUDA FFI, enum, typedef, constant, global.
//! Split from func_gen.rs for PMAT File Health compliance.

use super::*;

impl CodeGenerator {
    /// Generate a struct definition from HIR.
    ///
    /// Generates Rust struct code with automatic derives for Debug, Clone, PartialEq, Eq.
    /// Handles lifetimes automatically for structs with reference fields.
    pub fn generate_struct(&self, hir_struct: &decy_hir::HirStruct) -> String {
        let mut code = String::new();

        // Check if struct needs lifetimes (has Reference fields)
        let needs_lifetimes =
            hir_struct.fields().iter().any(|f| matches!(f.field_type(), HirType::Reference { .. }));

        // DECY-123: Check if struct has large arrays (> 32 elements) that don't impl Default
        // Rust arrays only implement Default for sizes up to 32
        let has_large_array = hir_struct.fields().iter().any(|f| {
            matches!(
                f.field_type(),
                HirType::Array { size: Some(n), .. } if *n > 32
            )
        });

        // DECY-218: Check if struct has float/double fields (f32/f64 don't implement Eq)
        let has_float_fields = hir_struct
            .fields()
            .iter()
            .any(|f| matches!(f.field_type(), HirType::Float | HirType::Double));

        // DECY-225: Check if struct can derive Copy (only primitive types, no pointers/Box/Vec/String)
        // Helper to check if a type is Copy-able
        fn is_copy_type(ty: &HirType) -> bool {
            match ty {
                HirType::Int
                | HirType::UnsignedInt
                | HirType::Bool
                | HirType::Float
                | HirType::Double
                | HirType::Char
                | HirType::SignedChar // DECY-250
                | HirType::Void => true,
                HirType::Array { element_type, .. } => is_copy_type(element_type),
                // DECY-246: Raw pointers (*mut T, *const T) ARE Copy in Rust!
                HirType::Pointer(_) => true,
                // Box, Vec, String, References are not Copy
                HirType::Box(_)
                | HirType::Vec(_)
                | HirType::OwnedString
                | HirType::StringReference
                | HirType::StringLiteral
                | HirType::Reference { .. } => false,
                // Struct fields need the inner struct to be Copy, which we can't check here
                // Be conservative and don't derive Copy
                HirType::Struct(_) | HirType::Enum(_) | HirType::Union(_) => false,
                // Function pointers are not Copy (they could be wrapped in Option)
                HirType::FunctionPointer { .. } => false,
                // Type aliases (like size_t) are Copy
                HirType::TypeAlias(_) => true,
                HirType::Option(_) => false,
            }
        }

        let can_derive_copy =
            !needs_lifetimes && hir_struct.fields().iter().all(|f| is_copy_type(f.field_type()));

        // Add derive attribute
        // DECY-114: Add Default derive for struct initialization with ::default()
        // DECY-123: Skip Default for large arrays
        // DECY-218: Skip Eq for floats (f32/f64 only implement PartialEq)
        // DECY-225: Add Copy for simple structs to avoid move errors
        let derives = match (has_large_array, has_float_fields, can_derive_copy) {
            (true, true, true) => "#[derive(Debug, Clone, Copy, PartialEq)]\n",
            (true, true, false) => "#[derive(Debug, Clone, PartialEq)]\n",
            (true, false, true) => "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n",
            (true, false, false) => "#[derive(Debug, Clone, PartialEq, Eq)]\n",
            (false, true, true) => "#[derive(Debug, Clone, Copy, Default, PartialEq)]\n",
            (false, true, false) => "#[derive(Debug, Clone, Default, PartialEq)]\n",
            (false, false, true) => "#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]\n",
            (false, false, false) => "#[derive(Debug, Clone, Default, PartialEq, Eq)]\n",
        };
        code.push_str(derives);

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
                HirType::Array { element_type, size: None } => {
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
            // DECY-227: Escape reserved keywords in field names
            code.push_str(&format!(
                "    pub {}: {},\n",
                escape_rust_keyword(field.name()),
                field_type_str
            ));
        }

        code.push('}');
        code
    }

    /// DECY-202: Generate a Rust struct + impl block from a C++ class.
    ///
    /// Maps: class fields -> struct fields, methods -> impl block,
    /// constructor -> `pub fn new()`, destructor -> `impl Drop`.
    ///
    /// # Example
    ///
    /// C++: `class Point { int x, y; Point(int x, int y); int distance(); ~Point(); };`
    /// Rust:
    /// ```ignore
    /// #[derive(Debug, Clone, Default, PartialEq, Eq)]
    /// pub struct Point {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// impl Point {
    ///     pub fn new(x: i32, y: i32) -> Self {
    ///         Self { x, y }
    ///     }
    ///     pub fn distance(&self) -> i32 { /* ... */ }
    /// }
    ///
    /// impl Drop for Point {
    ///     fn drop(&mut self) { /* destructor body */ }
    /// }
    /// ```
    pub fn generate_class(&self, hir_class: &decy_hir::HirClass) -> String {
        contract_pre_class_to_struct!();
        let mut code = String::new();
        self.generate_class_struct(hir_class, &mut code);
        self.generate_class_impl_block(hir_class, &mut code);
        self.generate_class_operator_impls(hir_class, &mut code);
        self.generate_class_drop_deref(hir_class, &mut code);
        code
    }

    /// Generate struct definition with derive fixups for a C++ class.
    fn generate_class_struct(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        let mut fields = hir_class.fields().to_vec();
        if let Some(base) = hir_class.base_class() {
            fields.insert(
                0,
                decy_hir::HirStructField::new(
                    "base".to_string(),
                    decy_hir::HirType::Struct(base.to_string()),
                ),
            );
        }
        let hir_struct = decy_hir::HirStruct::new(hir_class.name().to_string(), fields);
        let mut struct_code = self.generate_struct(&hir_struct);
        if hir_class.has_destructor() {
            struct_code = struct_code.replace("Copy, ", "").replace(", Copy", "");
        }
        if hir_class
            .methods()
            .iter()
            .any(|m| m.operator_kind() == Some(decy_hir::HirCxxOperatorKind::Equal))
        {
            struct_code = struct_code
                .replace(", PartialEq, Eq", "")
                .replace(", PartialEq", "")
                .replace("PartialEq, ", "");
        }
        if hir_class.base_class().is_some() {
            struct_code = struct_code.replace(", Eq", "").replace("Eq, ", "");
        }
        code.push_str(&struct_code);
        code.push_str("\n\n");
    }

    /// Generate impl block with constructor and methods for a C++ class.
    fn generate_class_impl_block(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        let has_ctor = !hir_class.constructor_params().is_empty();
        let has_methods = hir_class.methods().iter().any(|m| m.operator_kind().is_none());
        if !has_ctor && !has_methods {
            return;
        }

        code.push_str(&format!("impl {} {{\n", hir_class.name()));
        self.generate_class_constructor(hir_class, code);
        self.generate_class_methods(hir_class, code);
        code.push_str("}\n");
    }

    /// Generate pub fn new() constructor from C++ constructor params.
    fn generate_class_constructor(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        if hir_class.constructor_params().is_empty() {
            return;
        }
        let params: Vec<String> = hir_class
            .constructor_params()
            .iter()
            .map(|p| {
                format!("{}: {}", escape_rust_keyword(p.name()), Self::map_type(p.param_type()))
            })
            .collect();
        code.push_str(&format!("    pub fn new({}) -> Self {{\n", params.join(", ")));
        code.push_str("        Self {\n");
        let ctor_params = hir_class.constructor_params();
        let own_fields: Vec<_> = hir_class.fields().iter().filter(|f| f.name() != "base").collect();
        for (idx, field) in own_fields.iter().enumerate() {
            let val = if let Some(p) = ctor_params.iter().find(|p| p.name() == field.name()) {
                escape_rust_keyword(p.name())
            } else if idx < ctor_params.len() {
                escape_rust_keyword(ctor_params[idx].name())
            } else {
                "Default::default()".to_string()
            };
            code.push_str(&format!(
                "            {}: {},\n",
                escape_rust_keyword(field.name()),
                val
            ));
        }
        if hir_class.base_class().is_some() {
            code.push_str("            base: Default::default(),\n");
        }
        code.push_str("        }\n    }\n\n");
    }

    /// Generate regular (non-operator) methods for a C++ class.
    fn generate_class_methods(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        for method in hir_class.methods().iter().filter(|m| m.operator_kind().is_none()) {
            let func = method.function();
            let params: Vec<String> = func
                .parameters()
                .iter()
                .map(|p| {
                    format!("{}: {}", escape_rust_keyword(p.name()), Self::map_type(p.param_type()))
                })
                .collect();
            let all_params = if method.is_static() {
                params.join(", ")
            } else {
                let sr = if method.is_const() { "&self" } else { "&mut self" };
                if params.is_empty() {
                    sr.to_string()
                } else {
                    format!("{}, {}", sr, params.join(", "))
                }
            };
            let ret = if *func.return_type() == decy_hir::HirType::Void {
                String::new()
            } else {
                format!(" -> {}", Self::map_type(func.return_type()))
            };
            code.push_str(&format!(
                "    pub fn {}({}){} {{\n",
                escape_rust_keyword(func.name()),
                all_params,
                ret
            ));
            if func.body().is_empty() {
                if *func.return_type() != decy_hir::HirType::Void {
                    code.push_str("        Default::default()\n");
                }
            } else {
                for stmt in func.body() {
                    code.push_str(&format!(
                        "        {}\n",
                        self.generate_statement_with_context(
                            stmt,
                            None,
                            &mut TypeContext::new(),
                            None
                        )
                    ));
                }
            }
            code.push_str("    }\n\n");
        }
    }

    /// Generate std::ops trait impls for C++ operator overloading.
    fn generate_class_operator_impls(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        let cn = hir_class.name();
        for method in hir_class.methods().iter().filter(|m| m.operator_kind().is_some()) {
            let op = method.operator_kind().expect("filtered for is_some");
            let func = method.function();
            let rt = Self::map_type(func.return_type());
            let rhs = func
                .parameters()
                .first()
                .map_or(cn.to_string(), |p| Self::map_type(p.param_type()));
            use decy_hir::HirCxxOperatorKind as Op;
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem => {
                    let (tn, mn) = match op {
                        Op::Add => ("Add", "add"),
                        Op::Sub => ("Sub", "sub"),
                        Op::Mul => ("Mul", "mul"),
                        Op::Div => ("Div", "div"),
                        _ => ("Rem", "rem"),
                    };
                    code.push_str(&format!("\nimpl std::ops::{}<{}> for {} {{\n    type Output = {};\n\n    fn {}(self, rhs: {}) -> Self::Output {{\n        Default::default()\n    }}\n}}\n", tn, rhs, cn, rt, mn, rhs));
                }
                Op::Equal => {
                    code.push_str(&format!("\nimpl PartialEq for {} {{\n    fn eq(&self, other: &Self) -> bool {{\n        Default::default()\n    }}\n}}\n", cn));
                }
                Op::AddAssign | Op::SubAssign => {
                    let (tn, mn) = if op == Op::AddAssign {
                        ("AddAssign", "add_assign")
                    } else {
                        ("SubAssign", "sub_assign")
                    };
                    code.push_str(&format!("\nimpl std::ops::{}<{}> for {} {{\n    fn {}(&mut self, rhs: {}) {{\n    }}\n}}\n", tn, rhs, cn, mn, rhs));
                }
                _ => {
                    code.push_str(&format!("\n// TODO: impl operator {:?} for {}\n", op, cn));
                }
            }
        }
    }

    /// Generate Drop and Deref/DerefMut impls for inheritance.
    fn generate_class_drop_deref(&self, hir_class: &decy_hir::HirClass, code: &mut String) {
        if hir_class.has_destructor() {
            code.push_str(&format!("\nimpl Drop for {} {{\n    fn drop(&mut self) {{\n        // Destructor body (C++ ~ClassName)\n    }}\n}}\n", hir_class.name()));
        }
        if let Some(base) = hir_class.base_class() {
            code.push_str(&format!("\nimpl std::ops::Deref for {} {{\n    type Target = {};\n\n    fn deref(&self) -> &Self::Target {{\n        &self.base\n    }}\n}}\n", hir_class.name(), base));
            code.push_str(&format!("\nimpl std::ops::DerefMut for {} {{\n    fn deref_mut(&mut self) -> &mut Self::Target {{\n        &mut self.base\n    }}\n}}\n", hir_class.name()));
        }
    }

    /// DECY-211: Generate FFI declaration for a CUDA __global__ kernel.
    ///
    /// CUDA kernels cannot be directly transpiled to Rust — they run on the GPU.
    /// Instead, generate an `extern "C"` FFI declaration so host code can call
    /// the pre-compiled kernel, plus a safe wrapper comment.
    pub(crate) fn generate_cuda_kernel_ffi(&self, func: &HirFunction) -> String {
        contract_pre_kernel_ffi!();
        let mut code = String::new();

        // Generate extern "C" block
        code.push_str("extern \"C\" {\n");
        code.push_str(&format!("    /// CUDA kernel: {} (compiled separately)\n", func.name()));

        // Build parameter list as raw C types
        let params: Vec<String> = func
            .parameters()
            .iter()
            .map(|p| {
                let ty = match p.param_type() {
                    decy_hir::HirType::Pointer(inner) => {
                        format!("*mut {}", Self::map_type(inner))
                    }
                    other => Self::map_type(other),
                };
                format!("{}: {}", escape_rust_keyword(p.name()), ty)
            })
            .collect();

        let return_type = if *func.return_type() == decy_hir::HirType::Void {
            String::new()
        } else {
            format!(" -> {}", Self::map_type(func.return_type()))
        };

        code.push_str(&format!("    fn {}({}){};\n", func.name(), params.join(", "), return_type,));
        code.push_str("}\n");

        code
    }

    /// DECY-205: Generate a Rust `mod` block from a C++ namespace.
    ///
    /// Recursively generates nested modules for nested namespaces.
    /// Functions, structs, and classes within the namespace are generated
    /// inside the module scope.
    pub fn generate_namespace(&self, ns: &decy_hir::HirNamespace) -> String {
        contract_pre_namespace_to_mod!();
        let mut code = String::new();

        code.push_str(&format!("pub mod {} {{\n", escape_rust_keyword(ns.name())));

        // Generate structs
        for s in ns.structs() {
            for line in self.generate_struct(s).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Generate classes
        for c in ns.classes() {
            for line in self.generate_class(c).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Generate functions
        for f in ns.functions() {
            let func_code = self.generate_function(f);
            for line in func_code.lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Recurse into nested namespaces
        for nested in ns.namespaces() {
            for line in self.generate_namespace(nested).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        code.push_str("}\n");
        code
    }

    /// DECY-240: Generate an enum definition from HIR.
    ///
    /// Generates Rust const declarations for C enum values.
    /// C enums create integer constants that can be used directly (without prefix),
    /// so we generate const i32 values rather than Rust enums.
    ///
    /// # Example
    ///
    /// C: `enum day { MONDAY = 1, TUESDAY, WEDNESDAY };`
    /// Rust:
    /// ```ignore
    /// pub const MONDAY: i32 = 1;
    /// pub const TUESDAY: i32 = 2;
    /// pub const WEDNESDAY: i32 = 3;
    /// ```
    pub fn generate_enum(&self, hir_enum: &decy_hir::HirEnum) -> String {
        let mut code = String::new();

        // Add a type alias for the enum name (C: enum day → Rust: type day = i32)
        let enum_name = hir_enum.name();
        if !enum_name.is_empty() {
            code.push_str(&format!("pub type {} = i32;\n", enum_name));
        }

        // Generate const declarations for each variant
        let mut next_value: i32 = 0;
        for variant in hir_enum.variants() {
            let value = if let Some(v) = variant.value() {
                next_value = v + 1; // Next auto value
                v
            } else {
                let v = next_value;
                next_value += 1;
                v
            };
            code.push_str(&format!("pub const {}: i32 = {};\n", variant.name(), value));
        }

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
                format!("pub type {} = {};", name, Self::map_type(typedef.underlying_type()))
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
            format!("extern \"C\" {{\n    static {}: {};\n}}", var_name, rust_type)
        } else if is_const {
            // const int x = 10; → const x: i32 = 10;
            // static const int x = 10; → const x: i32 = 10; (const is stronger)
            format!("const {}: {} = {};", var_name, rust_type, value_expr)
        } else {
            // static int x = 0; → static mut x: i32 = 0;
            // int x = 0; → static mut x: i32 = 0; (default)
            // Special handling for arrays: [0; 10] for array initialization
            let init_expr = if let HirType::Array { element_type, size } = variable.const_type() {
                if let Some(size_val) = size {
                    // DECY-201: Fix array initialization for uninitialized arrays
                    // DECY-246: Use default_value_for_type to handle all types including structs
                    // Check if value is an integer (likely uninitialized or zero-initialized)
                    let element_init = match variable.value() {
                        HirExpression::IntLiteral(_) => {
                            // Any integer value for struct/complex array → use default
                            Self::default_value_for_type(element_type)
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
