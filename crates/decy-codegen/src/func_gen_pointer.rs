    fn is_parameter_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        // Check if the parameter is used in any assignment statements
        for stmt in func.body() {
            if self.statement_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

    fn is_parameter_deref_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_deref_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

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
            HirStatement::If { then_block, else_block, .. } => {
                then_block.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
            }
            _ => false,
        }
    }

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
