    pub fn generate_function_with_lifetimes(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
    ) -> String {
        self.generate_function_with_lifetimes_and_structs(func, sig, &[], &[], &[], &[], &[])
    }

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
