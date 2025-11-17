//! Pattern matching generation from tag checks (DECY-082).

use decy_hir::HirStatement;

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
    pub fn transform_tag_check(&self, _stmt: &HirStatement) -> String {
        todo!("DECY-082: Transform tag check to match expression")
    }
}

impl Default for PatternGenerator {
    fn default() -> Self {
        Self::new()
    }
}
