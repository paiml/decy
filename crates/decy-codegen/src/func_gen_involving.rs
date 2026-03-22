    fn function_uses_pointer_subtraction(&self, func: &HirFunction, var_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_uses_pointer_subtraction(stmt, var_name) {
                return true;
            }
        }
        false
    }

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
            HirStatement::If { condition, then_block, else_block, .. } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || then_block
                        .iter()
                        .any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    })
            }
            HirStatement::While { condition, body } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || body.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
            }
            HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
            }
            _ => false,
        }
    }

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
