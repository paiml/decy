    pub fn generate_signature(&self, func: &HirFunction) -> String {
        // DECY-076 GREEN: Generate lifetime annotations using LifetimeAnnotator
        use decy_ownership::lifetime_gen::LifetimeAnnotator;
        let lifetime_annotator = LifetimeAnnotator::new();
        let annotated_sig = lifetime_annotator.annotate_function(func);

        // DECY-241: Rename functions that conflict with Rust macros/keywords
        let safe_name = match func.name() {
            "write" => "c_write", // Conflicts with Rust's write! macro
            "read" => "c_read",   // Conflicts with Rust's read
            "type" => "c_type",   // Rust keyword
            "match" => "c_match", // Rust keyword
            "self" => "c_self",   // Rust keyword
            "in" => "c_in",       // Rust keyword
            name => name,
        };
        let mut sig = format!("fn {}", safe_name);

        // Add lifetime parameters if needed
        let lifetime_syntax = lifetime_annotator.generate_lifetime_syntax(&annotated_sig.lifetimes);
        sig.push_str(&lifetime_syntax);

        // DECY-096: Detect void* parameters for generic transformation
        use decy_analyzer::void_ptr_analysis::{TypeConstraint, VoidPtrAnalyzer};
        let void_analyzer = VoidPtrAnalyzer::new();
        let void_patterns = void_analyzer.analyze(func);

        // DECY-168: Only consider patterns with actual constraints/types as "real" void* usage
        // Empty body functions (stubs) will have patterns but no constraints
        let has_real_void_usage = void_patterns
            .iter()
            .any(|vp| !vp.constraints.is_empty() || !vp.inferred_types.is_empty());

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
                if skip_params.contains(&p.name) {
                    return None;
                }
                self.generate_signature_param(p, func, &graph, &void_patterns)
            })
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');

        // Generate return type
        self.append_signature_return_type(
            &mut sig,
            func,
            output_param_type.as_ref(),
            output_is_fallible,
            &annotated_sig,
        );

        sig
    }

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
