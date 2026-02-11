//! LLM-guided Rust code generation (DECY-099).
//!
//! Uses LLM to generate idiomatic Rust code guided by static analysis results.

use crate::context_builder::AnalysisContext;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during LLM code generation.
#[derive(Debug, Error)]
pub enum LlmError {
    /// Failed to create prompt from context
    #[error("Failed to create prompt: {0}")]
    PromptCreation(String),
    /// LLM API error
    #[error("LLM API error: {0}")]
    ApiError(String),
    /// Failed to parse LLM response
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    /// Generated code is invalid
    #[error("Generated code is invalid: {0}")]
    InvalidCode(String),
}

/// Result of LLM code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Generated Rust code
    pub code: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Reasoning for the generated code
    pub reasoning: String,
    /// Any warnings or suggestions
    pub warnings: Vec<String>,
}

/// Prompt template for LLM code generation.
#[derive(Debug, Clone)]
pub struct CodegenPrompt {
    /// C source code
    pub c_source: String,
    /// Analysis context (ownership, lifetimes, locks)
    pub context: AnalysisContext,
    /// Additional instructions
    pub instructions: String,
}

impl CodegenPrompt {
    /// Create a new codegen prompt.
    pub fn new(c_source: &str, context: AnalysisContext) -> Self {
        Self {
            c_source: c_source.to_string(),
            context,
            instructions: String::new(),
        }
    }

    /// Set additional instructions.
    pub fn with_instructions(mut self, instructions: &str) -> Self {
        self.instructions = instructions.to_string();
        self
    }

    /// Render the prompt as a string for LLM input.
    pub fn render(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("# C to Rust Transpilation Task\n\n");
        prompt.push_str("## Source C Code\n```c\n");
        prompt.push_str(&self.c_source);
        prompt.push_str("\n```\n\n");

        // Add analysis context
        prompt.push_str("## Static Analysis Context\n");
        if let Ok(context_json) = serde_json::to_string_pretty(&self.context) {
            prompt.push_str("```json\n");
            prompt.push_str(&context_json);
            prompt.push_str("\n```\n\n");
        }

        // Add ownership information summary
        for func in &self.context.functions {
            if !func.ownership.is_empty() {
                prompt.push_str(&format!("### Function: {}\n", func.name));
                prompt.push_str("Ownership analysis:\n");
                for (var, info) in &func.ownership {
                    prompt.push_str(&format!(
                        "- `{}`: {} (confidence: {:.0}%)\n",
                        var,
                        info.kind,
                        info.confidence * 100.0
                    ));
                }
                prompt.push('\n');
            }
        }

        if !self.instructions.is_empty() {
            prompt.push_str("## Additional Instructions\n");
            prompt.push_str(&self.instructions);
            prompt.push_str("\n\n");
        }

        prompt.push_str("## Task\n");
        prompt.push_str("Generate idiomatic, safe Rust code that is functionally equivalent to the C code above.\n");
        prompt.push_str(
            "Use the static analysis context to guide ownership and borrowing decisions.\n",
        );

        prompt
    }
}

/// LLM code generator.
#[derive(Debug)]
pub struct LlmCodegen {
    /// Model identifier
    model: String,
}

impl LlmCodegen {
    /// Create a new LLM code generator.
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }

    /// Generate Rust code from C source with analysis context.
    ///
    /// Note: This is a stub for research purposes. Actual LLM integration
    /// would require API credentials and network access.
    pub fn generate(&self, _prompt: &CodegenPrompt) -> Result<GeneratedCode, LlmError> {
        // In a real implementation, this would call the LLM API
        Err(LlmError::ApiError(format!(
            "LLM API not configured for model: {}",
            self.model
        )))
    }

    /// Parse raw LLM response into generated code.
    ///
    /// Supports two formats:
    /// 1. Markdown code blocks with ```rust ... ```
    /// 2. JSON with { "code": "...", "confidence": ..., ... }
    pub fn parse_response(&self, response: &str) -> Result<GeneratedCode, LlmError> {
        // Try JSON format first
        if let Ok(generated) = serde_json::from_str::<GeneratedCode>(response.trim()) {
            return Ok(generated);
        }

        // Try to extract code from markdown code blocks
        if let Some(code) = Self::extract_rust_code_block(response) {
            // Extract reasoning from text after code block
            let reasoning = Self::extract_reasoning(response);

            return Ok(GeneratedCode {
                code,
                confidence: 0.8, // Default confidence for markdown format
                reasoning,
                warnings: Vec::new(),
            });
        }

        Err(LlmError::ParseError(
            "No valid Rust code found in response".to_string(),
        ))
    }

    /// Extract Rust code from markdown code block.
    fn extract_rust_code_block(response: &str) -> Option<String> {
        // Look for ```rust or just ```
        let markers = ["```rust", "```"];

        for marker in markers {
            if let Some(start) = response.find(marker) {
                let code_start = start + marker.len();
                // Skip newline after marker
                let code_start = response[code_start..]
                    .find('\n')
                    .map(|i| code_start + i + 1)
                    .unwrap_or(code_start);

                // Find closing ```
                if let Some(end) = response[code_start..].find("```") {
                    let code = response[code_start..code_start + end].trim();
                    if !code.is_empty() {
                        return Some(code.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract reasoning text from response (text after code block).
    fn extract_reasoning(response: &str) -> String {
        // Find last ``` and get text after it
        if let Some(last_fence) = response.rfind("```") {
            let after = &response[last_fence + 3..];
            let reasoning = after.trim();
            if !reasoning.is_empty() {
                return reasoning.to_string();
            }
        }
        "Generated from C source".to_string()
    }

    /// Validate generated code (basic syntax check).
    ///
    /// Performs a basic syntactic validation:
    /// - Checks for balanced braces
    /// - Checks for fn keyword
    /// - Checks for basic syntax patterns
    pub fn validate_code(&self, code: &str) -> Result<(), LlmError> {
        // Check for balanced braces
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();

        if open_braces != close_braces {
            return Err(LlmError::InvalidCode(format!(
                "Unbalanced braces: {} open, {} close",
                open_braces, close_braces
            )));
        }

        // Check for balanced parentheses
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();

        if open_parens != close_parens {
            return Err(LlmError::InvalidCode(format!(
                "Unbalanced parentheses: {} open, {} close",
                open_parens, close_parens
            )));
        }

        // Check for basic function structure
        if code.contains("fn ") {
            // Looks like it has a function - basic check passed
            return Ok(());
        }

        // Allow simple expressions/statements too
        if !code.trim().is_empty() {
            return Ok(());
        }

        Err(LlmError::InvalidCode("Empty code".to_string()))
    }
}

impl Default for LlmCodegen {
    fn default() -> Self {
        Self::new("claude-3-sonnet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_block() {
        let response = "Here's the code:\n```rust\nfn main() {}\n```\nDone!";
        let code = LlmCodegen::extract_rust_code_block(response);
        assert!(code.is_some());
        assert!(code.unwrap().contains("fn main"));
    }

    // ========================================================================
    // CodegenPrompt tests
    // ========================================================================

    #[test]
    fn prompt_new_default_instructions_empty() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("int x = 5;", ctx);
        assert_eq!(prompt.c_source, "int x = 5;");
        assert!(prompt.instructions.is_empty());
    }

    #[test]
    fn prompt_with_instructions() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("int x;", ctx).with_instructions("Use safe Rust only");
        assert_eq!(prompt.instructions, "Use safe Rust only");
    }

    #[test]
    fn prompt_render_contains_c_source() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("int main() { return 0; }", ctx);
        let rendered = prompt.render();
        assert!(rendered.contains("int main() { return 0; }"));
        assert!(rendered.contains("# C to Rust Transpilation Task"));
        assert!(rendered.contains("## Source C Code"));
    }

    #[test]
    fn prompt_render_contains_instructions_when_set() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt =
            CodegenPrompt::new("void f();", ctx).with_instructions("Prefer Box over raw ptrs");
        let rendered = prompt.render();
        assert!(rendered.contains("## Additional Instructions"));
        assert!(rendered.contains("Prefer Box over raw ptrs"));
    }

    #[test]
    fn prompt_render_no_instructions_section_when_empty() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("void f();", ctx);
        let rendered = prompt.render();
        assert!(!rendered.contains("## Additional Instructions"));
    }

    #[test]
    fn prompt_render_includes_ownership_info() {
        use std::collections::HashMap;
        use crate::context_builder::{FunctionContext, OwnershipInfo};

        let mut ownership = HashMap::new();
        ownership.insert(
            "ptr".to_string(),
            OwnershipInfo {
                kind: "owning".to_string(),
                confidence: 0.95,
                reason: "malloc detected".to_string(),
            },
        );

        let ctx = AnalysisContext {
            functions: vec![FunctionContext {
                name: "alloc_data".to_string(),
                c_signature: "void* alloc_data()".to_string(),
                ownership,
                lifetimes: vec![],
                lock_mappings: HashMap::new(),
            }],
        };
        let prompt = CodegenPrompt::new("void* alloc_data() { return malloc(8); }", ctx);
        let rendered = prompt.render();
        assert!(rendered.contains("### Function: alloc_data"));
        assert!(rendered.contains("`ptr`: owning"));
        assert!(rendered.contains("95%"));
    }

    #[test]
    fn prompt_render_skips_functions_with_no_ownership() {
        use std::collections::HashMap;
        use crate::context_builder::FunctionContext;

        let ctx = AnalysisContext {
            functions: vec![FunctionContext {
                name: "simple".to_string(),
                c_signature: "int simple()".to_string(),
                ownership: HashMap::new(),
                lifetimes: vec![],
                lock_mappings: HashMap::new(),
            }],
        };
        let prompt = CodegenPrompt::new("int simple() { return 0; }", ctx);
        let rendered = prompt.render();
        assert!(!rendered.contains("### Function: simple"));
    }

    #[test]
    fn prompt_render_contains_task_section() {
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("int x;", ctx);
        let rendered = prompt.render();
        assert!(rendered.contains("## Task"));
        assert!(rendered.contains("Generate idiomatic, safe Rust code"));
    }

    // ========================================================================
    // LlmCodegen tests
    // ========================================================================

    #[test]
    fn llm_codegen_new() {
        let codegen = LlmCodegen::new("test-model");
        let debug = format!("{:?}", codegen);
        assert!(debug.contains("test-model"));
    }

    #[test]
    fn llm_codegen_default() {
        let codegen = LlmCodegen::default();
        let debug = format!("{:?}", codegen);
        assert!(debug.contains("claude-3-sonnet"));
    }

    #[test]
    fn llm_codegen_generate_returns_api_error() {
        let codegen = LlmCodegen::new("gpt-4");
        let ctx = AnalysisContext {
            functions: vec![],
        };
        let prompt = CodegenPrompt::new("int x;", ctx);
        let result = codegen.generate(&prompt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LlmError::ApiError(_)));
        assert!(err.to_string().contains("gpt-4"));
    }

    // ========================================================================
    // parse_response tests
    // ========================================================================

    #[test]
    fn parse_response_json_format() {
        let codegen = LlmCodegen::new("test");
        let json = r#"{"code": "fn main() {}", "confidence": 0.95, "reasoning": "simple", "warnings": []}"#;
        let result = codegen.parse_response(json).unwrap();
        assert_eq!(result.code, "fn main() {}");
        assert!((result.confidence - 0.95).abs() < 0.01);
        assert_eq!(result.reasoning, "simple");
    }

    #[test]
    fn parse_response_markdown_rust_block() {
        let codegen = LlmCodegen::new("test");
        let response = "Here is the code:\n```rust\nfn add(a: i32, b: i32) -> i32 { a + b }\n```\nThis adds two numbers.";
        let result = codegen.parse_response(response).unwrap();
        assert!(result.code.contains("fn add"));
        assert!((result.confidence - 0.8).abs() < 0.01);
        assert!(result.reasoning.contains("adds two numbers"));
    }

    #[test]
    fn parse_response_markdown_plain_block() {
        let codegen = LlmCodegen::new("test");
        let response = "Code:\n```\nlet x: i32 = 42;\n```\n";
        let result = codegen.parse_response(response).unwrap();
        assert!(result.code.contains("let x: i32 = 42"));
    }

    #[test]
    fn parse_response_no_code_returns_error() {
        let codegen = LlmCodegen::new("test");
        let response = "I cannot generate code for this.";
        let result = codegen.parse_response(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::ParseError(_)));
    }

    #[test]
    fn parse_response_empty_code_block_returns_error() {
        let codegen = LlmCodegen::new("test");
        let response = "```rust\n\n```";
        let result = codegen.parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn parse_response_no_reasoning_after_block() {
        let codegen = LlmCodegen::new("test");
        let response = "```rust\nfn main() {}\n```";
        let result = codegen.parse_response(response).unwrap();
        assert_eq!(result.reasoning, "Generated from C source");
    }

    // ========================================================================
    // validate_code tests
    // ========================================================================

    #[test]
    fn validate_code_balanced_with_fn() {
        let codegen = LlmCodegen::new("test");
        assert!(codegen.validate_code("fn main() { let x = 1; }").is_ok());
    }

    #[test]
    fn validate_code_unbalanced_braces() {
        let codegen = LlmCodegen::new("test");
        let result = codegen.validate_code("fn main() {");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("braces"));
    }

    #[test]
    fn validate_code_unbalanced_parens() {
        let codegen = LlmCodegen::new("test");
        let result = codegen.validate_code("fn main(");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("parentheses"));
    }

    #[test]
    fn validate_code_empty() {
        let codegen = LlmCodegen::new("test");
        let result = codegen.validate_code("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Empty"));
    }

    #[test]
    fn validate_code_whitespace_only() {
        let codegen = LlmCodegen::new("test");
        let result = codegen.validate_code("   \n  \t  ");
        assert!(result.is_err());
    }

    #[test]
    fn validate_code_expression_no_fn() {
        let codegen = LlmCodegen::new("test");
        // Non-empty, balanced, but no fn keyword — still passes (allows expressions)
        assert!(codegen.validate_code("let x = 42;").is_ok());
    }

    // ========================================================================
    // extract_reasoning tests
    // ========================================================================

    #[test]
    fn extract_reasoning_with_text_after_fence() {
        let response = "```rust\nfn main() {}\n```\nThis is a simple main function.";
        let reasoning = LlmCodegen::extract_reasoning(response);
        assert!(reasoning.contains("simple main function"));
    }

    #[test]
    fn extract_reasoning_no_text_after_fence() {
        let response = "```rust\nfn main() {}\n```";
        let reasoning = LlmCodegen::extract_reasoning(response);
        assert_eq!(reasoning, "Generated from C source");
    }

    #[test]
    fn extract_reasoning_no_fences() {
        let response = "Just some text without code blocks.";
        let reasoning = LlmCodegen::extract_reasoning(response);
        assert_eq!(reasoning, "Generated from C source");
    }

    // ========================================================================
    // LlmError Display tests
    // ========================================================================

    #[test]
    fn llm_error_display_variants() {
        let e1 = LlmError::PromptCreation("bad prompt".to_string());
        assert!(e1.to_string().contains("bad prompt"));

        let e2 = LlmError::ApiError("timeout".to_string());
        assert!(e2.to_string().contains("timeout"));

        let e3 = LlmError::ParseError("invalid json".to_string());
        assert!(e3.to_string().contains("invalid json"));

        let e4 = LlmError::InvalidCode("no braces".to_string());
        assert!(e4.to_string().contains("no braces"));
    }

    // ========================================================================
    // GeneratedCode serde tests
    // ========================================================================

    #[test]
    fn generated_code_serde_roundtrip() {
        let code = GeneratedCode {
            code: "fn main() {}".to_string(),
            confidence: 0.9,
            reasoning: "test".to_string(),
            warnings: vec!["warn1".to_string()],
        };
        let json = serde_json::to_string(&code).unwrap();
        let parsed: GeneratedCode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.code, "fn main() {}");
        assert_eq!(parsed.warnings.len(), 1);
    }

    #[test]
    fn generated_code_clone() {
        let code = GeneratedCode {
            code: "let x = 5;".to_string(),
            confidence: 0.8,
            reasoning: "simple".to_string(),
            warnings: vec![],
        };
        let cloned = code.clone();
        assert_eq!(code.code, cloned.code);
        assert_eq!(code.confidence, cloned.confidence);
    }
}
