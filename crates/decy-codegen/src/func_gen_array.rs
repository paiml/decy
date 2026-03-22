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
                name, initializer: Some(init_expr), ..
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

    pub(crate) fn is_any_malloc_or_calloc(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::Malloc { .. } => true,
            HirExpression::Calloc { .. } => true,
            HirExpression::FunctionCall { function, .. }
                if function == "malloc" || function == "calloc" =>
            {
                true
            }
            // DECY-220: Check through cast expressions (e.g., (int*)malloc(...))
            HirExpression::Cast { expr: inner, .. } => Self::is_any_malloc_or_calloc(inner),
            _ => false,
        }
    }

    fn is_malloc_call(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::FunctionCall { function, arguments, .. } if function == "malloc" => {
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

    fn is_array_allocation_size(size_expr: &HirExpression) -> bool {
        match size_expr {
            // n * sizeof(T) pattern - this is array allocation
            HirExpression::BinaryOp { op: decy_hir::BinaryOperator::Multiply, .. } => true,
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
