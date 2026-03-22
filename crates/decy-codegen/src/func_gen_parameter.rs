    fn generate_pointer_param(
        &self,
        name: &str,
        inner: &HirType,
        func: &HirFunction,
        void_patterns: &[decy_analyzer::void_ptr_analysis::VoidPtrInfo],
    ) -> String {
        use decy_analyzer::void_ptr_analysis::TypeConstraint;

        // DECY-096: void* param becomes generic &T or &mut T
        // DECY-168: Only apply generic transformation if we found an actual pattern
        // for this specific parameter WITH real constraints (from body analysis).
        // Otherwise keep as raw pointer *mut ().
        if matches!(inner, HirType::Void) {
            // Look for a void pattern specifically for this parameter
            // that has actual constraints (indicating real usage in body)
            let void_pattern = void_patterns.iter().find(|vp| {
                vp.param_name == name
                    && (!vp.constraints.is_empty()
                        || !vp.inferred_types.is_empty())
            });

            if let Some(pattern) = void_pattern {
                // Found actual usage pattern - apply generic transformation
                let is_mutable = pattern.constraints.contains(&TypeConstraint::Mutable);
                if is_mutable {
                    return format!("{}: &mut T", name);
                } else {
                    return format!("{}: &T", name);
                }
            } else {
                // DECY-168: No pattern with real constraints found - keep as raw pointer
                // This is important for stdlib stubs (realloc, memcpy, etc.)
                return format!("{}: *mut ()", name);
            }
        }
        // DECY-134: Check for string iteration pattern FIRST
        // char* with pointer arithmetic → slice instead of raw pointer
        if self.is_string_iteration_param(func, name) {
            // Transform to slice for safe string iteration
            let is_mutable = self.is_parameter_deref_modified(func, name);
            if is_mutable {
                return format!("{}: &mut [u8]", name);
            } else {
                return format!("{}: &[u8]", name);
            }
        }
        // DECY-123: Don't transform to reference if pointer arithmetic is used
        // (e.g., ptr = ptr + 1) - keep as raw pointer
        if self.uses_pointer_arithmetic(func, name) {
            // Keep as raw pointer - will need unsafe blocks
            // DECY-124: Add mut since the pointer is reassigned
            let inner_type = Self::map_type(inner);
            return format!("mut {}: *mut {}", name, inner_type);
        }
        // Transform pointer param to mutable reference
        // Check if the param is modified in the function body
        let is_mutable = self.is_parameter_deref_modified(func, name);
        let inner_type = Self::map_type(inner);
        if is_mutable {
            format!("{}: &mut {}", name, inner_type)
        } else {
            // Read-only pointer becomes immutable reference
            format!("{}: &{}", name, inner_type)
        }
    }

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

    fn generate_annotated_pointer_param(
        &self,
        name: &str,
        inner: &HirType,
        func: Option<&HirFunction>,
    ) -> String {
        // DECY-135: const char* → &str transformation
        if let Some(f) = func {
            if let Some(orig_param) =
                f.parameters().iter().find(|fp| fp.name() == name)
            {
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
