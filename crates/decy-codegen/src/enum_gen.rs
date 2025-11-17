//! Enum generation from tagged unions (DECY-081).

use decy_analyzer::tagged_union_analysis::TaggedUnionInfo;

/// Generator for Rust enums from C tagged unions.
pub struct EnumGenerator;

impl EnumGenerator {
    /// Create a new enum generator.
    pub fn new() -> Self {
        Self
    }

    /// Generate a Rust enum from tagged union info.
    ///
    /// # Arguments
    ///
    /// * `info` - Tagged union information from analysis
    ///
    /// # Returns
    ///
    /// Rust enum definition as a string
    pub fn generate_enum(&self, _info: &TaggedUnionInfo) -> String {
        todo!("DECY-081: Generate Rust enum from tagged union")
    }
}

impl Default for EnumGenerator {
    fn default() -> Self {
        Self::new()
    }
}
