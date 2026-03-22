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
                (format!("{} ", length_param), format!("{}.len() as i32 ", array_param)),
                (format!("{})", length_param), format!("{}.len() as i32)", array_param)),
                (format!("{},", length_param), format!("{}.len() as i32,", array_param)),
                (format!("{}]", length_param), format!("{}.len() as i32]", array_param)),
                (length_param.clone() + "}", array_param.clone() + ".len() as i32}"),
                (format!("{};", length_param), format!("{}.len() as i32;", array_param)),
            ];

            for (pattern, replacement) in patterns {
                result = result.replace(&pattern, &replacement);
            }
        }

        result
    }

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
