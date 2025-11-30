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
#[derive(Debug, Default)]
pub struct ContextBuilder {
    /// Functions being built
    functions: Vec<FunctionContext>,
}

impl ContextBuilder {
    /// Create a new context builder.
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    /// Build context from ownership and lifetime analysis results.
    pub fn build(&self) -> AnalysisContext {
        AnalysisContext {
            functions: self.functions.clone(),
        }
    }

    /// Add a function with its analysis results.
    pub fn add_function(&mut self, name: &str, c_signature: &str) -> &mut Self {
        self.functions.push(FunctionContext {
            name: name.to_string(),
            c_signature: c_signature.to_string(),
            ownership: HashMap::new(),
            lifetimes: Vec::new(),
            lock_mappings: HashMap::new(),
        });
        self
    }

    /// Add ownership inference for a variable.
    pub fn add_ownership(
        &mut self,
        function: &str,
        variable: &str,
        kind: &str,
        confidence: f64,
        reason: &str,
    ) -> &mut Self {
        if let Some(func) = self.functions.iter_mut().find(|f| f.name == function) {
            func.ownership.insert(
                variable.to_string(),
                OwnershipInfo {
                    kind: kind.to_string(),
                    confidence,
                    reason: reason.to_string(),
                },
            );
        }
        self
    }

    /// Add lifetime information.
    pub fn add_lifetime(
        &mut self,
        function: &str,
        variable: &str,
        scope_depth: usize,
        escapes: bool,
    ) -> &mut Self {
        if let Some(func) = self.functions.iter_mut().find(|f| f.name == function) {
            func.lifetimes.push(LifetimeInfo {
                variable: variable.to_string(),
                scope_depth,
                escapes,
                depends_on: Vec::new(),
            });
        }
        self
    }

    /// Add lock-to-data mapping.
    pub fn add_lock_mapping(
        &mut self,
        function: &str,
        lock: &str,
        protected_data: Vec<String>,
    ) -> &mut Self {
        if let Some(func) = self.functions.iter_mut().find(|f| f.name == function) {
            func.lock_mappings.insert(lock.to_string(), protected_data);
        }
        self
    }

    /// Serialize context to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let context = self.build();
        serde_json::to_string_pretty(&context)
    }
}
