//! Pattern matching generation from tag checks (DECY-082).

use decy_hir::{BinaryOperator, HirExpression, HirStatement};

/// Generator for Rust pattern matching from C tag checks.
pub struct PatternGenerator;

impl PatternGenerator {
    /// Create a new pattern generator.
    pub fn new() -> Self {
        Self
    }

    /// Transform C tag check (if statement) into Rust match expression.
    ///
    /// # Arguments
    ///
    /// * `stmt` - HIR if statement checking a tag field
    ///
    /// # Returns
    ///
    /// Rust match expression as a string, or empty string if not a tag check
    pub fn transform_tag_check(&self, stmt: &HirStatement) -> String {
        if let HirStatement::If {
            condition,
            then_block,
            else_block,
        } = stmt
        {
            // Check if this is a tag comparison
            if let Some((var_name, _tag_field, _tag_value)) = Self::extract_tag_check(condition) {
                let mut result = String::new();

                // Start match expression
                result.push_str(&format!("match {} {{\n", var_name));

                // Collect all arms from if-else-if chain
                let arms = self.collect_match_arms(stmt);

                for arm in arms {
                    result.push_str(&format!("    {},\n", arm));
                }

                result.push('}');
                return result;
            }
        }

        // Not a tag check - return empty
        String::new()
    }

    /// Extract tag check components from condition.
    /// Returns (variable_name, tag_field, tag_value) if valid tag check.
    fn extract_tag_check(condition: &HirExpression) -> Option<(String, String, String)> {
        if let HirExpression::BinaryOp { left, op, right } = condition {
            if !matches!(op, BinaryOperator::Equal) {
                return None;
            }

            // Left should be field access: v.tag
            if let HirExpression::FieldAccess { object, field } = &**left {
                if let HirExpression::Variable(var_name) = &**object {
                    // Right should be enum constant
                    if let HirExpression::Variable(tag_value) = &**right {
                        return Some((var_name.clone(), field.clone(), tag_value.clone()));
                    }
                }
            }
        }
        None
    }

    /// Collect all match arms from if-else-if chain.
    fn collect_match_arms(&self, stmt: &HirStatement) -> Vec<String> {
        let mut arms = Vec::new();
        self.collect_arms_recursive(stmt, &mut arms);
        arms
    }

    /// Recursively collect match arms from nested if statements.
    fn collect_arms_recursive(&self, stmt: &HirStatement, arms: &mut Vec<String>) {
        if let HirStatement::If {
            condition,
            then_block,
            else_block,
        } = stmt
        {
            if let Some((_var_name, _tag_field, tag_value)) = Self::extract_tag_check(condition) {
                // Generate match arm for this variant
                let variant_name = Self::capitalize_tag_value(&tag_value);
                let binding = Self::extract_union_field_binding(then_block);

                let arm_body = self.generate_arm_body(then_block);

                let arm = if let Some(field) = binding {
                    format!("Value::{}({}) => {}", variant_name, field, arm_body)
                } else {
                    format!("Value::{} => {}", variant_name, arm_body)
                };

                arms.push(arm);

                // Process else block
                if let Some(else_stmts) = else_block {
                    if else_stmts.len() == 1 {
                        // Check if it's another tag check (else-if)
                        if matches!(else_stmts[0], HirStatement::If { .. }) {
                            self.collect_arms_recursive(&else_stmts[0], arms);
                            return;
                        }
                    }

                    // Not another tag check - generate default arm
                    let else_body = self.generate_block_body(else_stmts);
                    arms.push(format!("_ => {}", else_body));
                }
            }
        }
    }

    /// Capitalize tag value to PascalCase variant name.
    fn capitalize_tag_value(tag_value: &str) -> String {
        let parts: Vec<String> = tag_value
            .split('_')
            .filter(|s| !s.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let rest: String = chars.collect::<String>().to_lowercase();
                        first.to_uppercase().collect::<String>() + &rest
                    }
                }
            })
            .collect();

        if parts.is_empty() {
            tag_value.to_string()
        } else {
            parts.join("")
        }
    }

    /// Extract union field name from then block if it accesses v.data.field.
    fn extract_union_field_binding(then_block: &[HirStatement]) -> Option<String> {
        for stmt in then_block {
            if let Some(field) = Self::find_union_field_in_stmt(stmt) {
                return Some(field);
            }
        }
        None
    }

    /// Find union field access in a statement.
    fn find_union_field_in_stmt(stmt: &HirStatement) -> Option<String> {
        match stmt {
            HirStatement::Return(Some(expr)) => Self::find_union_field_in_expr(expr),
            HirStatement::Expression(expr) => Self::find_union_field_in_expr(expr),
            _ => None,
        }
    }

    /// Find union field access in an expression (v.data.field_name).
    fn find_union_field_in_expr(expr: &HirExpression) -> Option<String> {
        if let HirExpression::FieldAccess { object, field } = expr {
            // Check if object is v.data
            if let HirExpression::FieldAccess {
                object: _inner_obj,
                field: inner_field,
            } = &**object
            {
                if inner_field == "data" {
                    return Some(field.clone());
                }
            }
        }
        None
    }

    /// Generate arm body from then block.
    fn generate_arm_body(&self, then_block: &[HirStatement]) -> String {
        if then_block.len() == 1 {
            if let HirStatement::Return(Some(expr)) = &then_block[0] {
                // Simple return - just return the expression
                let field = Self::find_union_field_in_expr(expr);
                if let Some(f) = field {
                    return format!("return {}", f);
                }
                return "return /* value */".to_string();
            }
        }

        self.generate_block_body(then_block)
    }

    /// Generate body for a block of statements.
    fn generate_block_body(&self, stmts: &[HirStatement]) -> String {
        if stmts.is_empty() {
            return "{}".to_string();
        }

        if stmts.len() == 1 {
            if matches!(&stmts[0], HirStatement::Return(Some(_))) {
                return "/* return value */".to_string();
            }
        }

        "{ /* block */ }".to_string()
    }
}

impl Default for PatternGenerator {
    fn default() -> Self {
        Self::new()
    }
}
