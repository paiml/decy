//! Context builder for LLM prompts.
//!
//! Builds structured JSON context from static analysis results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete analysis context for LLM consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// Functions in the file with their analysis results
    pub functions: Vec<FunctionContext>,
}

/// Per-function analysis context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContext {
    /// Function name
    pub name: String,
    /// Original C signature
    pub c_signature: String,
    /// Ownership inferences for variables
    pub ownership: HashMap<String, OwnershipInfo>,
    /// Lifetime relationships
    pub lifetimes: Vec<LifetimeInfo>,
    /// Lock-to-data mappings
    pub lock_mappings: HashMap<String, Vec<String>>,
}

/// Serializable ownership information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    /// Ownership kind: "owning", "immutable_borrow", "mutable_borrow", "unknown"
    pub kind: String,
    /// Confidence score 0.0-1.0
    pub confidence: f64,
    /// Human-readable reasoning
    pub reason: String,
}

/// Serializable lifetime information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeInfo {
    /// Variable name
    pub variable: String,
    /// Scope depth
    pub scope_depth: usize,
    /// Whether variable escapes its scope
    pub escapes: bool,
    /// Dependent lifetimes (other variables this depends on)
    pub depends_on: Vec<String>,
}

/// Builder for LLM context from analysis results.
pub struct ContextBuilder;

impl ContextBuilder {
    /// Create a new context builder.
    pub fn new() -> Self {
        todo!("DECY-098: Implement ContextBuilder::new")
    }

    /// Build context from ownership and lifetime analysis results.
    pub fn build(&self) -> AnalysisContext {
        todo!("DECY-098: Implement ContextBuilder::build")
    }

    /// Add a function with its analysis results.
    pub fn add_function(&mut self, _name: &str, _c_signature: &str) -> &mut Self {
        todo!("DECY-098: Implement ContextBuilder::add_function")
    }

    /// Add ownership inference for a variable.
    pub fn add_ownership(
        &mut self,
        _function: &str,
        _variable: &str,
        _kind: &str,
        _confidence: f64,
        _reason: &str,
    ) -> &mut Self {
        todo!("DECY-098: Implement ContextBuilder::add_ownership")
    }

    /// Add lifetime information.
    pub fn add_lifetime(
        &mut self,
        _function: &str,
        _variable: &str,
        _scope_depth: usize,
        _escapes: bool,
    ) -> &mut Self {
        todo!("DECY-098: Implement ContextBuilder::add_lifetime")
    }

    /// Add lock-to-data mapping.
    pub fn add_lock_mapping(
        &mut self,
        _function: &str,
        _lock: &str,
        _protected_data: Vec<String>,
    ) -> &mut Self {
        todo!("DECY-098: Implement ContextBuilder::add_lock_mapping")
    }

    /// Serialize context to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        todo!("DECY-098: Implement ContextBuilder::to_json")
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
