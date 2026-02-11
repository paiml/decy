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

// ============================================================================
// TEST 11: Render prompt with ownership data for multiple functions
// ============================================================================

#[test]
fn test_render_prompt_multiple_functions_with_ownership() {
    let mut builder = ContextBuilder::new();
    builder
        .add_function("init", "void init(int* buf, size_t len)")
        .add_ownership("init", "buf", "mutable_borrow", 0.95, "Modified in loop")
        .add_function("process", "void process(int* data)")
        .add_ownership("process", "data", "owning", 0.8, "Freed at end");

    let prompt = CodegenPrompt::new(
        "void init(int* buf, size_t len) {}\nvoid process(int* data) {}",
        builder.build(),
    );

    let rendered = prompt.render();

    // Should include both function headings
    assert!(rendered.contains("Function: init"), "Got: {}", rendered);
    assert!(rendered.contains("Function: process"), "Got: {}", rendered);
    // Should include ownership details
    assert!(rendered.contains("mutable_borrow"), "Got: {}", rendered);
    assert!(rendered.contains("owning"), "Got: {}", rendered);
    assert!(rendered.contains("95%"), "Got: {}", rendered);
    assert!(rendered.contains("80%"), "Got: {}", rendered);
}

// ============================================================================
// TEST 12: Render prompt with instructions
// ============================================================================

#[test]
fn test_render_prompt_with_instructions_section() {
    let context = ContextBuilder::new().build();
    let prompt = CodegenPrompt::new("int x = 5;", context)
        .with_instructions("Prefer safe Rust patterns. Avoid raw pointers.");

    let rendered = prompt.render();

    assert!(
        rendered.contains("Additional Instructions"),
        "Got: {}",
        rendered
    );
    assert!(
        rendered.contains("Prefer safe Rust patterns"),
        "Got: {}",
        rendered
    );
    assert!(rendered.contains("Task"), "Got: {}", rendered);
}

// ============================================================================
// TEST 13: Render prompt with empty functions (no ownership data)
// ============================================================================

#[test]
fn test_render_prompt_functions_without_ownership() {
    let mut builder = ContextBuilder::new();
    builder.add_function("empty_func", "void empty_func()");

    let prompt = CodegenPrompt::new("void empty_func() {}", builder.build());

    let rendered = prompt.render();

    // Should NOT include function ownership heading since ownership is empty
    assert!(
        !rendered.contains("Function: empty_func"),
        "Should skip functions with no ownership data, Got: {}",
        rendered
    );
    // Should still have the task section
    assert!(rendered.contains("Task"), "Got: {}", rendered);
}

// ============================================================================
// TEST 14: Validate empty code returns error
// ============================================================================

#[test]
fn test_validate_empty_code() {
    let codegen = LlmCodegen::new("test-model");
    let result = codegen.validate_code("");
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("Empty"), "Got: {}", err);
}

// ============================================================================
// TEST 15: Validate whitespace-only code returns error
// ============================================================================

#[test]
fn test_validate_whitespace_only_code() {
    let codegen = LlmCodegen::new("test-model");
    let result = codegen.validate_code("   \n\t  \n  ");
    assert!(result.is_err());
}

// ============================================================================
// TEST 16: Validate non-fn non-empty code succeeds
// ============================================================================

#[test]
fn test_validate_non_fn_non_empty_code() {
    let codegen = LlmCodegen::new("test-model");
    // Code without `fn ` but non-empty should pass (line 248)
    let result = codegen.validate_code("let x: i32 = 42;");
    assert!(result.is_ok());
}

// ============================================================================
// TEST 17: Validate unbalanced parentheses
// ============================================================================

#[test]
fn test_validate_unbalanced_parens() {
    let codegen = LlmCodegen::new("test-model");
    let result = codegen.validate_code("fn add(a: i32, b: i32 { a + b }");
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("parentheses"), "Got: {}", err);
}

// ============================================================================
// TEST 18: Generate returns API error stub
// ============================================================================

#[test]
fn test_generate_returns_api_error() {
    let codegen = LlmCodegen::new("test-model");
    let context = ContextBuilder::new().build();
    let prompt = CodegenPrompt::new("int x;", context);

    let result = codegen.generate(&prompt);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("test-model"), "Got: {}", err);
}

// ============================================================================
// TEST 19: Parse response with plain code block (no rust marker)
// ============================================================================

#[test]
fn test_parse_response_plain_code_block() {
    let codegen = LlmCodegen::new("test-model");

    let response = "Here's the result:\n```\nlet x: i32 = 42;\n```\nDone.";

    let result = codegen.parse_response(response);
    assert!(result.is_ok());
    let generated = result.unwrap();
    assert!(generated.code.contains("let x"), "Got: {}", generated.code);
    assert!((generated.confidence - 0.8).abs() < 0.01);
}

// ============================================================================
// TEST 20: Parse response extracts reasoning after code
// ============================================================================

#[test]
fn test_parse_response_extracts_reasoning() {
    let codegen = LlmCodegen::new("test-model");

    let response =
        "```rust\nfn hello() {}\n```\nThis converts the C function to idiomatic Rust.";

    let result = codegen.parse_response(response);
    assert!(result.is_ok());
    let generated = result.unwrap();
    assert!(
        generated.reasoning.contains("idiomatic Rust"),
        "Got reasoning: {}",
        generated.reasoning
    );
}

// ============================================================================
// TEST 21: Render prompt with JSON context section
// ============================================================================

#[test]
fn test_render_prompt_includes_json_context() {
    let mut builder = ContextBuilder::new();
    builder.add_function("test_fn", "int test_fn()");

    let prompt = CodegenPrompt::new("int test_fn() { return 0; }", builder.build());
    let rendered = prompt.render();

    // Should have JSON context block
    assert!(
        rendered.contains("Static Analysis Context"),
        "Got: {}",
        rendered
    );
    assert!(rendered.contains("```json"), "Got: {}", rendered);
    assert!(rendered.contains("test_fn"), "Got: {}", rendered);
}

// ============================================================================
// TEST 22: Validate unbalanced braces (lines 224-227)
// ============================================================================

#[test]
fn test_validate_unbalanced_braces() {
    let codegen = LlmCodegen::new("test-model");
    let result = codegen.validate_code("fn foo() { { }"); // 2 open, 1 close
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("braces"), "Got: {}", err);
}

// ============================================================================
// TEST 23: Parse response with empty code block (lines 191-192)
// ============================================================================

#[test]
fn test_parse_response_empty_code_block() {
    let codegen = LlmCodegen::new("test-model");
    // Code block with empty content between markers
    let response = "```rust\n\n```\nSome reasoning";
    let result = codegen.parse_response(response);
    // Should fail since code is empty
    assert!(result.is_err(), "Should fail on empty code block");
}

// ============================================================================
// TEST 24: Parse response with no closing fence (extract_reasoning fallback)
// ============================================================================

#[test]
fn test_parse_response_reasoning_fallback() {
    let codegen = LlmCodegen::new("test-model");
    // Response with code block but nothing after last ```
    let response = "```rust\nfn main() {}\n```";
    let result = codegen.parse_response(response);
    assert!(result.is_ok());
    let generated = result.unwrap();
    // Reasoning should be the fallback since there's nothing after last ```
    assert_eq!(generated.reasoning, "Generated from C source");
}
