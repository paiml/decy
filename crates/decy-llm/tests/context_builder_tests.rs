//! Tests for LLM context builder (DECY-098).
//!
//! Verifies that static analysis results are properly serialized
//! as structured JSON for LLM prompts.

use decy_llm::{AnalysisContext, ContextBuilder, FunctionContext};

// ============================================================================
// TEST 1: Create empty context builder
// ============================================================================

#[test]
fn test_create_context_builder() {
    let builder = ContextBuilder::new();
    let context = builder.build();
    assert!(context.functions.is_empty(), "Empty builder should have no functions");
}

// ============================================================================
// TEST 2: Add function to context
// ============================================================================

#[test]
fn test_add_function_to_context() {
    let mut builder = ContextBuilder::new();
    builder.add_function("process", "void process(int* data, size_t len)");

    let context = builder.build();
    assert_eq!(context.functions.len(), 1);
    assert_eq!(context.functions[0].name, "process");
    assert_eq!(context.functions[0].c_signature, "void process(int* data, size_t len)");
}

// ============================================================================
// TEST 3: Add ownership inference
// ============================================================================

#[test]
fn test_add_ownership_inference() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("transfer", "void transfer(int* dest, int* src)")
        .add_ownership("transfer", "dest", "mutable_borrow", 0.95, "Mutated via assignment")
        .add_ownership("transfer", "src", "immutable_borrow", 0.9, "Read-only access");

    let context = builder.build();
    let func = &context.functions[0];

    assert!(func.ownership.contains_key("dest"));
    assert!(func.ownership.contains_key("src"));

    let dest_ownership = &func.ownership["dest"];
    assert_eq!(dest_ownership.kind, "mutable_borrow");
    assert!((dest_ownership.confidence - 0.95).abs() < 0.01);

    let src_ownership = &func.ownership["src"];
    assert_eq!(src_ownership.kind, "immutable_borrow");
}

// ============================================================================
// TEST 4: Add lifetime information
// ============================================================================

#[test]
fn test_add_lifetime_info() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("create", "int* create()")
        .add_lifetime("create", "result", 0, true);  // Escapes function

    let context = builder.build();
    let func = &context.functions[0];

    assert_eq!(func.lifetimes.len(), 1);
    assert_eq!(func.lifetimes[0].variable, "result");
    assert_eq!(func.lifetimes[0].scope_depth, 0);
    assert!(func.lifetimes[0].escapes);
}

// ============================================================================
// TEST 5: Add lock-to-data mapping
// ============================================================================

#[test]
fn test_add_lock_mapping() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("sync_update", "void sync_update()")
        .add_lock_mapping("sync_update", "counter_mutex", vec!["counter".to_string(), "total".to_string()]);

    let context = builder.build();
    let func = &context.functions[0];

    assert!(func.lock_mappings.contains_key("counter_mutex"));
    let protected = &func.lock_mappings["counter_mutex"];
    assert!(protected.contains(&"counter".to_string()));
    assert!(protected.contains(&"total".to_string()));
}

// ============================================================================
// TEST 6: Serialize to JSON
// ============================================================================

#[test]
fn test_serialize_to_json() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("example", "int example(int* ptr)")
        .add_ownership("example", "ptr", "owning", 0.85, "Allocated via malloc");

    let json = builder.to_json().expect("JSON serialization failed");

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

    // Should have functions array
    assert!(parsed["functions"].is_array());
    assert_eq!(parsed["functions"][0]["name"], "example");
    assert_eq!(parsed["functions"][0]["ownership"]["ptr"]["kind"], "owning");
}

// ============================================================================
// TEST 7: Multiple functions
// ============================================================================

#[test]
fn test_multiple_functions() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("init", "void init()")
        .add_function("cleanup", "void cleanup()")
        .add_function("process", "int process(int* data)");

    let context = builder.build();
    assert_eq!(context.functions.len(), 3);

    let names: Vec<&str> = context.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"init"));
    assert!(names.contains(&"cleanup"));
    assert!(names.contains(&"process"));
}

// ============================================================================
// TEST 8: Complex function with all analysis types
// ============================================================================

#[test]
fn test_complex_function_context() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("thread_safe_update", "void thread_safe_update(Counter* c)")
        .add_ownership("thread_safe_update", "c", "mutable_borrow", 0.9, "Modified under lock")
        .add_lifetime("thread_safe_update", "c", 0, false)
        .add_lock_mapping("thread_safe_update", "mutex", vec!["c".to_string()]);

    let context = builder.build();
    let func = &context.functions[0];

    // All three analysis types should be present
    assert!(!func.ownership.is_empty());
    assert!(!func.lifetimes.is_empty());
    assert!(!func.lock_mappings.is_empty());
}

// ============================================================================
// TEST 9: JSON schema compliance
// ============================================================================

#[test]
fn test_json_schema_structure() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("test_fn", "void test_fn(int* ptr)")
        .add_ownership("test_fn", "ptr", "immutable_borrow", 0.8, "Read only")
        .add_lifetime("test_fn", "ptr", 1, false)
        .add_lock_mapping("test_fn", "lock", vec!["data".to_string()]);

    let json = builder.to_json().expect("Serialization failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify schema structure
    let func = &parsed["functions"][0];

    // Function has required fields
    assert!(func["name"].is_string());
    assert!(func["c_signature"].is_string());
    assert!(func["ownership"].is_object());
    assert!(func["lifetimes"].is_array());
    assert!(func["lock_mappings"].is_object());

    // Ownership info has required fields
    let ownership = &func["ownership"]["ptr"];
    assert!(ownership["kind"].is_string());
    assert!(ownership["confidence"].is_number());
    assert!(ownership["reason"].is_string());

    // Lifetime info has required fields
    let lifetime = &func["lifetimes"][0];
    assert!(lifetime["variable"].is_string());
    assert!(lifetime["scope_depth"].is_number());
    assert!(lifetime["escapes"].is_boolean());
}

// ============================================================================
// TEST 10: Deserialization roundtrip
// ============================================================================

#[test]
fn test_json_roundtrip() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("roundtrip", "int roundtrip(int* x, int* y)")
        .add_ownership("roundtrip", "x", "mutable_borrow", 0.95, "Modified")
        .add_ownership("roundtrip", "y", "immutable_borrow", 0.88, "Read only")
        .add_lifetime("roundtrip", "x", 0, false)
        .add_lifetime("roundtrip", "y", 0, false);

    let json = builder.to_json().expect("Serialization failed");
    let deserialized: AnalysisContext = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(deserialized.functions.len(), 1);
    assert_eq!(deserialized.functions[0].name, "roundtrip");
    assert_eq!(deserialized.functions[0].ownership.len(), 2);
    assert_eq!(deserialized.functions[0].lifetimes.len(), 2);
}
