//! Transform code generation: annotated signatures, box/vec transforms.
//! Split from func_gen.rs for PMAT File Health compliance.

use super::*;
use decy_hir::HirStatement;
use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};

impl CodeGenerator {
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
        // DECY-241: Rename functions that conflict with Rust macros/keywords
        let safe_name = match sig.name.as_str() {
            "write" => "c_write",
            "read" => "c_read",
            "type" => "c_type",
            "match" => "c_match",
            "self" => "c_self",
            "in" => "c_in",
            name => name,
        };
        let mut result = format!("fn {}", safe_name);

        // DECY-084/085: Detect output parameters for transformation
        let (skip_output_params, output_param_types, output_is_fallible) =
            Self::detect_output_params(func);

        // DECY-072: Check if we have any non-slice reference parameters that need lifetimes
        // Slices have elided lifetimes and don't need explicit lifetime parameters
        let has_non_slice_references = sig.parameters.iter().any(|p| {
            match &p.param_type {
                AnnotatedType::Reference { inner, .. } => {
                    // Check if this is NOT a slice (slice = Reference to Array with size=None)
                    !matches!(&**inner, AnnotatedType::Simple(HirType::Array { size: None, .. }))
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
            .map(|p| self.generate_annotated_param(p, func))
            .collect();
        result.push_str(&params.join(", "));
        result.push(')');

        // Generate return type
        self.append_annotated_return_type(
            &mut result,
            sig,
            func,
            &output_param_types,
            output_is_fallible,
        );

        result
    }

    /// Detect output parameters from a function for signature transformation.
    /// Returns (skip_set, output_types, is_fallible).
    fn detect_output_params(
        func: Option<&HirFunction>,
    ) -> (std::collections::HashSet<String>, Vec<HirType>, bool) {
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let mut skip_output_params = std::collections::HashSet::new();
        let mut output_param_types: Vec<HirType> = Vec::new();
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
            let output_param_count =
                output_params.iter().filter(|op| op.kind == ParameterKind::Output).count();

            for op in &output_params {
                if op.kind == ParameterKind::Output {
                    // Heuristic: Only treat as output param if:
                    // 1. There are other input parameters (output is derived from inputs)
                    // 2. Or, the name suggests it's an output (result, out, output, ret, etc.)
                    // 3. DECY-085: Or, there are multiple output params (void func with multiple outs)
                    let is_output_name = Self::is_output_param_name(&op.name);

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

        (skip_output_params, output_param_types, output_is_fallible)
    }

    /// Check if a parameter name suggests it is an output parameter.
    fn is_output_param_name(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("result")
            || name_lower.contains("out")
            || name_lower.contains("ret")
            || name_lower == "len"
            || name_lower == "size"
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
    }

    /// Generate a single annotated parameter string.
    fn generate_annotated_param(
        &self,
        p: &decy_ownership::lifetime_gen::AnnotatedParameter,
        func: Option<&HirFunction>,
    ) -> String {
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
            let type_str = match &p.param_type {
                AnnotatedType::Reference { inner, mutable, .. } => {
                    if let AnnotatedType::Simple(HirType::Array { element_type, .. }) = &**inner {
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
            return format!("{}: {}", p.name, type_str);
        }

        // DECY-111: Transform pointer parameters to mutable references
        // DECY-123: Skip transformation if pointer arithmetic is used
        if let AnnotatedType::Simple(HirType::Pointer(inner)) = &p.param_type {
            return self.generate_annotated_pointer_param(&p.name, inner, func);
        }

        // DECY-196: Handle unsized array parameters → slice references
        if let AnnotatedType::Simple(HirType::Array { element_type, size: None }) = &p.param_type {
            let element_str = Self::map_type(element_type);
            return format!("{}: &mut [{}]", p.name, element_str);
        }

        // DECY-041: Add mut for all non-slice parameters to match C semantics
        format!("mut {}: {}", p.name, self.annotated_type_to_string(&p.param_type))
    }

    /// Generate an annotated pointer parameter (reference, raw pointer, slice, or &str).
    fn generate_annotated_pointer_param(
        &self,
        name: &str,
        inner: &HirType,
        func: Option<&HirFunction>,
    ) -> String {
        // DECY-135: const char* → &str transformation
        if let Some(f) = func {
            if let Some(orig_param) = f.parameters().iter().find(|fp| fp.name() == name) {
                if orig_param.is_const_char_pointer() {
                    return format!("mut {}: &str", name);
                }
            }
        }
        // DECY-134: Check for string iteration pattern FIRST
        if let Some(f) = func {
            if self.is_string_iteration_param(f, name) {
                let is_mutable = self.is_parameter_deref_modified(f, name);
                if is_mutable {
                    return format!("{}: &mut [u8]", name);
                } else {
                    return format!("{}: &[u8]", name);
                }
            }
        }
        // DECY-123: If we have function body access, check for pointer arithmetic
        if let Some(f) = func {
            if self.uses_pointer_arithmetic(f, name) {
                let inner_type = Self::map_type(inner);
                return format!("mut {}: *mut {}", name, inner_type);
            }
        }
        // DECY-168: void* parameters should stay as raw pointers
        if matches!(*inner, HirType::Void) {
            return format!("{}: *mut ()", name);
        }
        // Transform *mut T → &mut T for safety
        let inner_type = Self::map_type(inner);
        format!("{}: &mut {}", name, inner_type)
    }

    /// Append return type for annotated signature.
    fn append_annotated_return_type(
        &self,
        result: &mut String,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
        output_param_types: &[HirType],
        output_is_fallible: bool,
    ) {
        // Special handling for main function (DECY-AUDIT-001)
        let return_type_str = self.annotated_type_to_string(&sig.return_type);
        if sig.name == "main" && return_type_str == "i32" {
            return;
        }

        // DECY-084/085: Generate return type considering output parameters
        if !output_param_types.is_empty() {
            let out_type_str = if output_param_types.len() == 1 {
                Self::map_type(&output_param_types[0])
            } else {
                let type_strs: Vec<String> =
                    output_param_types.iter().map(Self::map_type).collect();
                format!("({})", type_strs.join(", "))
            };

            if output_is_fallible {
                result.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                result.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check for Vec return type (malloc'd array returns)
            if let Some(f) = func {
                if let Some(vec_element_type) = self.detect_vec_return(f) {
                    let element_type_str = Self::map_type(&vec_element_type);
                    result.push_str(&format!(" -> Vec<{}>", element_type_str));
                    return;
                }
            }
            // Add return type if not void
            if return_type_str != "()" {
                result.push_str(&format!(" -> {}", return_type_str));
            }
        }
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
            AnnotatedType::Reference { inner, mutable, lifetime } => {
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
            HirType::Bool => "    return false;".to_string(),
            HirType::Int => "    return 0;".to_string(),
            HirType::UnsignedInt => "    return 0;".to_string(), // DECY-158
            HirType::Float => "    return 0.0;".to_string(),
            HirType::Double => "    return 0.0;".to_string(),
            HirType::Char => "    return 0;".to_string(),
            HirType::SignedChar => "    return 0;".to_string(), // DECY-250
            HirType::Pointer(_) => "    return std::ptr::null_mut();".to_string(),
            HirType::Box(inner) => {
                format!("    return Box::new({});", Self::default_value_for_type(inner))
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
                    format!("    return [{}; {}];", Self::default_value_for_type(element_type), n)
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
            HirType::TypeAlias(name) => match name.as_str() {
                "size_t" => "    return 0usize;".to_string(),
                "ssize_t" | "ptrdiff_t" => "    return 0isize;".to_string(),
                _ => "    return 0;".to_string(),
            },
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
        // DECY-211: CUDA __global__ kernels -> extern "C" FFI wrapper
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Global) {
            return self.generate_cuda_kernel_ffi(func);
        }
        // DECY-211: CUDA __device__ functions -> comment noting device-only
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Device) {
            let sig = self.generate_signature(func);
            return format!(
                "// CUDA __device__ function — runs on GPU only, not transpiled\n// {}\n",
                sig
            );
        }

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
                            HirType::Reference { inner: inner.clone(), mutable: is_mutable },
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
        // DECY-221: CUDA kernel/device functions bypass normal codegen
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Global) {
            return self.generate_cuda_kernel_ffi(func);
        }
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Device) {
            let sig = self.generate_signature(func);
            return format!("// CUDA __device__ function — GPU only\n// {}\n", sig);
        }

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
        self.generate_function_with_lifetimes_and_structs(func, sig, &[], &[], &[], &[], &[])
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
    /// * `globals` - DECY-220/233: Global variable names and types for unsafe access and type inference
    #[allow(clippy::too_many_arguments)]
    pub fn generate_function_with_lifetimes_and_structs(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
        structs: &[decy_hir::HirStruct],
        all_functions: &[(String, Vec<HirType>)],
        slice_func_args: &[(String, Vec<(usize, usize)>)],
        string_iter_funcs: &[(String, Vec<(usize, bool)>)],
        globals: &[(String, HirType)],
    ) -> String {
        contract_pre_host_transpilation!();
        // DECY-221: CUDA kernel/device functions bypass normal codegen
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Global) {
            return self.generate_cuda_kernel_ffi(func);
        }
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Device) {
            let sig_str = self.generate_signature(func);
            return format!(
                "// CUDA __device__ function — runs on GPU only, not transpiled\n// {}\n",
                sig_str
            );
        }

        let mut code = String::new();

        // Generate signature with lifetimes
        // DECY-123: Pass function for pointer arithmetic detection
        code.push_str(&self.generate_annotated_signature_with_func(sig, Some(func)));
        code.push_str(" {\n");

        // DECY-041: Initialize type context with function parameters for pointer arithmetic
        let mut ctx = TypeContext::from_function(func);

        // DECY-220/233: Register global variables for unsafe access tracking and type inference
        for (name, var_type) in globals {
            ctx.add_global(name.clone());
            ctx.add_variable(name.clone(), var_type.clone());
        }

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
                            HirType::Reference { inner: inner.clone(), mutable: is_mutable },
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
    pub(crate) fn transform_vec_statement(
        &self,
        stmt: &HirStatement,
        candidate: &decy_analyzer::patterns::VecCandidate,
    ) -> HirStatement {
        match stmt {
            HirStatement::VariableDeclaration { name, var_type, initializer: _ } => {
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
            HirStatement::Assignment { target: _, value: _ } => {
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
                    self.box_transformer.transform_statement(stmt, box_candidate)
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
}
