//! Tests for LLM code generation (DECY-099).
//!
//! Verifies LLM-guided Rust generation with analysis context.

use decy_llm::{CodegenPrompt, ContextBuilder, LlmCodegen};

// ============================================================================
// TEST 1: Create codegen prompt
// ============================================================================

#[test]
fn test_create_codegen_prompt() {
    let context = ContextBuilder::new().build();
    let c_source = "int add(int a, int b) { return a + b; }";

    let prompt = CodegenPrompt::new(c_source, context);

    assert_eq!(prompt.c_source, c_source);
    assert!(prompt.instructions.is_empty());
}

// ============================================================================
// TEST 2: Add instructions to prompt
// ============================================================================

#[test]
fn test_prompt_with_instructions() {
    let context = ContextBuilder::new().build();
    let c_source = "void process(int* data) { *data = 0; }";

    let prompt = CodegenPrompt::new(c_source, context)
        .with_instructions("Generate safe Rust using references");

    assert!(prompt.instructions.contains("references"));
}

// ============================================================================
// TEST 3: Render prompt includes context
// ============================================================================

#[test]
fn test_render_prompt_includes_context() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("transfer", "void transfer(int* dest, int* src)")
        .add_ownership("transfer", "dest", "mutable_borrow", 0.9, "Modified");

    let prompt = CodegenPrompt::new(
        "void transfer(int* dest, int* src) { *dest = *src; }",
        builder.build(),
    );

    let rendered = prompt.render();

    // Should include C source
    assert!(rendered.contains("transfer"));
    // Should include ownership info
    assert!(rendered.contains("mutable_borrow") || rendered.contains("ownership"));
}

// ============================================================================
// TEST 4: Create LLM codegen
// ============================================================================

#[test]
fn test_create_llm_codegen() {
    let _codegen = LlmCodegen::new("test-model");
    // Just verify creation doesn't panic
}

// ============================================================================
// TEST 5: Parse valid LLM response
// ============================================================================

#[test]
fn test_parse_valid_response() {
    let codegen = LlmCodegen::new("test-model");

    let response = r#"
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

This is a simple addition function that takes two i32 parameters and returns their sum.
"#;

    let result = codegen.parse_response(response);
    assert!(result.is_ok());

    let generated = result.unwrap();
    assert!(generated.code.contains("fn add"));
    assert!(generated.code.contains("a + b"));
}

// ============================================================================
// TEST 6: Parse response with JSON format
// ============================================================================

#[test]
fn test_parse_json_response() {
    let codegen = LlmCodegen::new("test-model");

    let response = r#"
{
    "code": "fn add(a: i32, b: i32) -> i32 { a + b }",
    "confidence": 0.95,
    "reasoning": "Simple arithmetic conversion",
    "warnings": []
}
"#;

    let result = codegen.parse_response(response);
    assert!(result.is_ok());

    let generated = result.unwrap();
    assert!(generated.code.contains("fn add"));
    assert!((generated.confidence - 0.95).abs() < 0.01);
}

// ============================================================================
// TEST 7: Parse malformed response returns error
// ============================================================================

#[test]
fn test_parse_malformed_response() {
    let codegen = LlmCodegen::new("test-model");

    let response = "This response has no code at all, just random text.";

    let result = codegen.parse_response(response);
    assert!(result.is_err());
}

// ============================================================================
// TEST 8: Validate valid Rust code
// ============================================================================

#[test]
fn test_validate_valid_code() {
    let codegen = LlmCodegen::new("test-model");

    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";

    let result = codegen.validate_code(code);
    assert!(result.is_ok());
}

// ============================================================================
// TEST 9: Validate invalid Rust code returns error
// ============================================================================

#[test]
fn test_validate_invalid_code() {
    let codegen = LlmCodegen::new("test-model");

    let code = "fn add(a: i32, b: { a + b }"; // Missing return type

    let result = codegen.validate_code(code);
    assert!(result.is_err());
}

// ============================================================================
// TEST 10: Default model is claude-3-sonnet
// ============================================================================

#[test]
fn test_default_model() {
    let _codegen = LlmCodegen::default();
    // Should create without panic using default model
}
