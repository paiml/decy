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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_new_empty() {
        let builder = ContextBuilder::new();
        let ctx = builder.build();
        assert!(ctx.functions.is_empty());
    }

    #[test]
    fn builder_default() {
        let builder = ContextBuilder::default();
        let ctx = builder.build();
        assert!(ctx.functions.is_empty());
    }

    #[test]
    fn builder_add_function() {
        let mut builder = ContextBuilder::new();
        builder.add_function("process", "void process(int* data, int len)");
        let ctx = builder.build();
        assert_eq!(ctx.functions.len(), 1);
        assert_eq!(ctx.functions[0].name, "process");
        assert_eq!(
            ctx.functions[0].c_signature,
            "void process(int* data, int len)"
        );
    }

    #[test]
    fn builder_add_multiple_functions() {
        let mut builder = ContextBuilder::new();
        builder.add_function("foo", "void foo()");
        builder.add_function("bar", "int bar(int x)");
        let ctx = builder.build();
        assert_eq!(ctx.functions.len(), 2);
        assert_eq!(ctx.functions[0].name, "foo");
        assert_eq!(ctx.functions[1].name, "bar");
    }

    #[test]
    fn builder_add_ownership() {
        let mut builder = ContextBuilder::new();
        builder.add_function("alloc", "void* alloc()");
        builder.add_ownership("alloc", "ptr", "owning", 0.95, "malloc detected");
        let ctx = builder.build();
        let func = &ctx.functions[0];
        assert!(func.ownership.contains_key("ptr"));
        let info = &func.ownership["ptr"];
        assert_eq!(info.kind, "owning");
        assert!((info.confidence - 0.95).abs() < 0.01);
        assert_eq!(info.reason, "malloc detected");
    }

    #[test]
    fn builder_add_ownership_nonexistent_function() {
        let mut builder = ContextBuilder::new();
        builder.add_function("foo", "void foo()");
        // Adding ownership to a nonexistent function should be a no-op
        builder.add_ownership("nonexistent", "ptr", "owning", 0.9, "test");
        let ctx = builder.build();
        assert!(ctx.functions[0].ownership.is_empty());
    }

    #[test]
    fn builder_add_lifetime() {
        let mut builder = ContextBuilder::new();
        builder.add_function("borrow", "int* borrow(int* src)");
        builder.add_lifetime("borrow", "src", 1, false);
        let ctx = builder.build();
        let func = &ctx.functions[0];
        assert_eq!(func.lifetimes.len(), 1);
        assert_eq!(func.lifetimes[0].variable, "src");
        assert_eq!(func.lifetimes[0].scope_depth, 1);
        assert!(!func.lifetimes[0].escapes);
        assert!(func.lifetimes[0].depends_on.is_empty());
    }

    #[test]
    fn builder_add_lifetime_escaping() {
        let mut builder = ContextBuilder::new();
        builder.add_function("escape", "int* escape()");
        builder.add_lifetime("escape", "result", 0, true);
        let ctx = builder.build();
        assert!(ctx.functions[0].lifetimes[0].escapes);
    }

    #[test]
    fn builder_add_lifetime_nonexistent_function() {
        let mut builder = ContextBuilder::new();
        builder.add_function("foo", "void foo()");
        builder.add_lifetime("nonexistent", "var", 0, false);
        let ctx = builder.build();
        assert!(ctx.functions[0].lifetimes.is_empty());
    }

    #[test]
    fn builder_add_lock_mapping() {
        let mut builder = ContextBuilder::new();
        builder.add_function("sync", "void sync()");
        builder.add_lock_mapping(
            "sync",
            "mutex_a",
            vec!["counter".to_string(), "buffer".to_string()],
        );
        let ctx = builder.build();
        let func = &ctx.functions[0];
        assert!(func.lock_mappings.contains_key("mutex_a"));
        let protected = &func.lock_mappings["mutex_a"];
        assert_eq!(protected.len(), 2);
        assert!(protected.contains(&"counter".to_string()));
    }

    #[test]
    fn builder_add_lock_mapping_nonexistent_function() {
        let mut builder = ContextBuilder::new();
        builder.add_function("foo", "void foo()");
        builder.add_lock_mapping("nonexistent", "lock", vec!["data".to_string()]);
        let ctx = builder.build();
        assert!(ctx.functions[0].lock_mappings.is_empty());
    }

    #[test]
    fn builder_to_json() {
        let mut builder = ContextBuilder::new();
        builder.add_function("main", "int main()");
        builder.add_ownership("main", "buf", "mutable_borrow", 0.85, "mutation");
        let json = builder.to_json().unwrap();
        assert!(json.contains("\"name\": \"main\""));
        assert!(json.contains("\"buf\""));
        assert!(json.contains("mutable_borrow"));
    }

    #[test]
    fn builder_to_json_empty() {
        let builder = ContextBuilder::new();
        let json = builder.to_json().unwrap();
        assert!(json.contains("functions"));
    }

    #[test]
    fn builder_chaining() {
        let mut builder = ContextBuilder::new();
        builder
            .add_function("f", "void f(int* p)")
            .add_ownership("f", "p", "immutable_borrow", 0.9, "read-only")
            .add_lifetime("f", "p", 1, false)
            .add_lock_mapping("f", "mtx", vec!["shared".to_string()]);
        let ctx = builder.build();
        assert_eq!(ctx.functions.len(), 1);
        assert_eq!(ctx.functions[0].ownership.len(), 1);
        assert_eq!(ctx.functions[0].lifetimes.len(), 1);
        assert_eq!(ctx.functions[0].lock_mappings.len(), 1);
    }

    #[test]
    fn analysis_context_serde_roundtrip() {
        let mut builder = ContextBuilder::new();
        builder.add_function("test", "void test()");
        builder.add_ownership("test", "x", "owning", 0.8, "test reason");
        let ctx = builder.build();
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: AnalysisContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.functions.len(), 1);
        assert_eq!(parsed.functions[0].name, "test");
    }

    #[test]
    fn function_context_default_fields() {
        let mut builder = ContextBuilder::new();
        builder.add_function("empty", "void empty()");
        let ctx = builder.build();
        let func = &ctx.functions[0];
        assert!(func.ownership.is_empty());
        assert!(func.lifetimes.is_empty());
        assert!(func.lock_mappings.is_empty());
    }

    #[test]
    fn ownership_info_serde() {
        let info = OwnershipInfo {
            kind: "owning".to_string(),
            confidence: 0.99,
            reason: "test".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: OwnershipInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.kind, "owning");
        assert!((parsed.confidence - 0.99).abs() < 0.01);
    }

    #[test]
    fn lifetime_info_serde() {
        let info = LifetimeInfo {
            variable: "p".to_string(),
            scope_depth: 2,
            escapes: true,
            depends_on: vec!["q".to_string()],
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: LifetimeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.variable, "p");
        assert!(parsed.escapes);
        assert_eq!(parsed.depends_on, vec!["q".to_string()]);
    }
}
