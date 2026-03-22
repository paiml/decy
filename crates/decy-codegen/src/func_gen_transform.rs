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

    fn transform_vec_statement(
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
