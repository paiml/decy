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
}
