    fn statement_uses_null_comparison(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::If { condition, then_block, else_block, .. } => {
                // Check condition for NULL comparison
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                // Recursively check nested statements
                then_block.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
                    })
            }
            HirStatement::While { condition, body, .. } => {
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                body.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            HirStatement::For { condition, body, .. } => {
                if let Some(cond) = condition {
                    if self.expression_compares_to_null(cond, var_name) {
                        return true;
                    }
                }
                body.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            _ => false,
        }
    }

    fn expression_compares_to_null(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right } => {
                if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                    // Check: var == 0 or var != 0
                    if let HirExpression::Variable(name) = &**left {
                        if name == var_name
                            && matches!(
                                **right,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                    // Check: 0 == var or 0 != var
                    if let HirExpression::Variable(name) = &**right {
                        if name == var_name
                            && matches!(
                                **left,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                }
                // Recursively check nested expressions (e.g., in logical AND/OR)
                self.expression_compares_to_null(left, var_name)
                    || self.expression_compares_to_null(right, var_name)
            }
            _ => false,
        }
    }
