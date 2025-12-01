//! Dataset export module for HuggingFace publishing
//!
//! This module provides functionality to export oracle patterns to various formats
//! suitable for machine learning training and HuggingFace dataset publishing.
//!
//! # Formats
//!
//! - **JSONL**: JSON Lines format for instruction tuning (ChatML, Alpaca)
//! - **Parquet**: Columnar format for efficient storage and Arrow compatibility
//!
//! # Example
//!
//! ```ignore
//! use decy_oracle::dataset::{DatasetExporter, ExportFormat};
//!
//! let exporter = DatasetExporter::new();
//! exporter.export_jsonl("patterns.jsonl")?;
//! exporter.export_parquet("patterns.parquet")?;
//! ```

use crate::bootstrap::{get_bootstrap_patterns, BootstrapPattern};
use crate::error::OracleError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single training example for the oracle dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    /// Error code (e.g., "E0308")
    pub error_code: String,
    /// Decision type (e.g., "type_coercion", "unsafe_deref")
    pub decision: String,
    /// Fix diff showing the transformation
    pub fix_diff: String,
    /// Human-readable description
    pub description: String,
    /// Source of the pattern (bootstrap, training, imported)
    pub source: String,
    /// Whether this pattern has been verified by compilation
    pub verified: bool,
    /// Success count (how many times this pattern worked)
    pub success_count: u32,
    /// Failure count (how many times this pattern failed)
    pub failure_count: u32,
}

impl TrainingExample {
    /// Create from a bootstrap pattern
    pub fn from_bootstrap(pattern: &BootstrapPattern) -> Self {
        Self {
            error_code: pattern.error_code.to_string(),
            decision: pattern.decision.to_string(),
            fix_diff: pattern.fix_diff.to_string(),
            description: pattern.description.to_string(),
            source: "bootstrap".to_string(),
            verified: true, // Bootstrap patterns are pre-verified
            success_count: 0,
            failure_count: 0,
        }
    }
}

/// ChatML format for instruction tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMLMessage {
    pub role: String,
    pub content: String,
}

/// ChatML conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMLConversation {
    pub messages: Vec<ChatMLMessage>,
}

/// Alpaca format for instruction tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaExample {
    pub instruction: String,
    pub input: String,
    pub output: String,
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON Lines - one JSON object per line
    Jsonl,
    /// ChatML format for conversational fine-tuning
    ChatML,
    /// Alpaca format for instruction tuning
    Alpaca,
    /// Apache Parquet for Arrow/HuggingFace datasets
    Parquet,
}

/// Dataset exporter for oracle patterns
pub struct DatasetExporter {
    examples: Vec<TrainingExample>,
}

impl DatasetExporter {
    /// Create a new exporter with bootstrap patterns
    pub fn new() -> Self {
        let bootstrap_patterns = get_bootstrap_patterns();
        let examples = bootstrap_patterns
            .iter()
            .map(TrainingExample::from_bootstrap)
            .collect();

        Self { examples }
    }

    /// Create an empty exporter
    pub fn empty() -> Self {
        Self {
            examples: Vec::new(),
        }
    }

    /// Add a training example
    pub fn add_example(&mut self, example: TrainingExample) {
        self.examples.push(example);
    }

    /// Get all examples
    pub fn examples(&self) -> &[TrainingExample] {
        &self.examples
    }

    /// Number of examples
    pub fn len(&self) -> usize {
        self.examples.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    /// Export to JSONL format (one JSON object per line)
    pub fn export_jsonl(&self, path: impl AsRef<Path>) -> Result<usize, OracleError> {
        let path = path.as_ref();
        let mut output = String::new();

        for example in &self.examples {
            let json = serde_json::to_string(example).map_err(|e| {
                OracleError::ExportError(format!("Failed to serialize example: {}", e))
            })?;
            output.push_str(&json);
            output.push('\n');
        }

        std::fs::write(path, &output).map_err(|e| {
            OracleError::ExportError(format!(
                "Failed to write JSONL to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(self.examples.len())
    }

    /// Export to ChatML format for conversational fine-tuning
    pub fn export_chatml(&self, path: impl AsRef<Path>) -> Result<usize, OracleError> {
        let path = path.as_ref();
        let mut output = String::new();

        for example in &self.examples {
            let conversation = ChatMLConversation {
                messages: vec![
                    ChatMLMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Fix the following Rust compilation error:\n\nError: {} - {}\n\nContext:\n{}",
                            example.error_code, example.description, example.fix_diff.lines().filter(|l| l.starts_with('-')).collect::<Vec<_>>().join("\n")
                        ),
                    },
                    ChatMLMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "Apply this fix ({}):\n\n{}",
                            example.decision,
                            example.fix_diff.lines().filter(|l| l.starts_with('+')).collect::<Vec<_>>().join("\n")
                        ),
                    },
                ],
            };

            let json = serde_json::to_string(&conversation).map_err(|e| {
                OracleError::ExportError(format!("Failed to serialize ChatML: {}", e))
            })?;
            output.push_str(&json);
            output.push('\n');
        }

        std::fs::write(path, &output).map_err(|e| {
            OracleError::ExportError(format!(
                "Failed to write ChatML to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(self.examples.len())
    }

    /// Export to Alpaca format for instruction tuning
    pub fn export_alpaca(&self, path: impl AsRef<Path>) -> Result<usize, OracleError> {
        let path = path.as_ref();
        let mut output = String::new();

        for example in &self.examples {
            let alpaca = AlpacaExample {
                instruction: format!(
                    "Fix the Rust compilation error {} ({}).",
                    example.error_code, example.description
                ),
                input: example
                    .fix_diff
                    .lines()
                    .filter(|l| l.starts_with('-'))
                    .map(|l| l.trim_start_matches('-').trim())
                    .collect::<Vec<_>>()
                    .join("\n"),
                output: example
                    .fix_diff
                    .lines()
                    .filter(|l| l.starts_with('+'))
                    .map(|l| l.trim_start_matches('+').trim())
                    .collect::<Vec<_>>()
                    .join("\n"),
            };

            let json = serde_json::to_string(&alpaca).map_err(|e| {
                OracleError::ExportError(format!("Failed to serialize Alpaca: {}", e))
            })?;
            output.push_str(&json);
            output.push('\n');
        }

        std::fs::write(path, &output).map_err(|e| {
            OracleError::ExportError(format!(
                "Failed to write Alpaca to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(self.examples.len())
    }

    /// Export to Parquet format using alimentar
    #[cfg(feature = "dataset")]
    pub fn export_parquet(&self, path: impl AsRef<Path>) -> Result<usize, OracleError> {
        use alimentar::ArrowDataset;
        use arrow::array::{BooleanArray, StringArray, UInt32Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        let path = path.as_ref();

        // Build Arrow arrays from examples
        let error_codes: StringArray = self
            .examples
            .iter()
            .map(|e| Some(e.error_code.as_str()))
            .collect();
        let decisions: StringArray = self
            .examples
            .iter()
            .map(|e| Some(e.decision.as_str()))
            .collect();
        let fix_diffs: StringArray = self
            .examples
            .iter()
            .map(|e| Some(e.fix_diff.as_str()))
            .collect();
        let descriptions: StringArray = self
            .examples
            .iter()
            .map(|e| Some(e.description.as_str()))
            .collect();
        let sources: StringArray = self
            .examples
            .iter()
            .map(|e| Some(e.source.as_str()))
            .collect();
        let verified: BooleanArray = self.examples.iter().map(|e| Some(e.verified)).collect();
        let success_counts: UInt32Array =
            self.examples.iter().map(|e| Some(e.success_count)).collect();
        let failure_counts: UInt32Array =
            self.examples.iter().map(|e| Some(e.failure_count)).collect();

        // Create schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("error_code", DataType::Utf8, false),
            Field::new("decision", DataType::Utf8, false),
            Field::new("fix_diff", DataType::Utf8, false),
            Field::new("description", DataType::Utf8, false),
            Field::new("source", DataType::Utf8, false),
            Field::new("verified", DataType::Boolean, false),
            Field::new("success_count", DataType::UInt32, false),
            Field::new("failure_count", DataType::UInt32, false),
        ]));

        // Create record batch
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(error_codes),
                Arc::new(decisions),
                Arc::new(fix_diffs),
                Arc::new(descriptions),
                Arc::new(sources),
                Arc::new(verified),
                Arc::new(success_counts),
                Arc::new(failure_counts),
            ],
        )
        .map_err(|e| OracleError::ExportError(format!("Failed to create RecordBatch: {}", e)))?;

        // Create dataset from batch
        let dataset = ArrowDataset::from_batch(batch)
            .map_err(|e| OracleError::ExportError(format!("Failed to create Arrow dataset: {}", e)))?;

        dataset
            .to_parquet(path)
            .map_err(|e| OracleError::ExportError(format!("Failed to write Parquet: {}", e)))?;

        Ok(self.examples.len())
    }

    /// Export to Parquet format (stub when alimentar not available)
    #[cfg(not(feature = "dataset"))]
    pub fn export_parquet(&self, _path: impl AsRef<Path>) -> Result<usize, OracleError> {
        Err(OracleError::ExportError(
            "Parquet export requires the 'dataset' feature. Build with --features dataset"
                .to_string(),
        ))
    }

    /// Get dataset statistics
    pub fn stats(&self) -> DatasetStats {
        let mut by_error_code: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut by_decision: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut by_source: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut verified_count = 0;

        for example in &self.examples {
            *by_error_code.entry(example.error_code.clone()).or_default() += 1;
            *by_decision.entry(example.decision.clone()).or_default() += 1;
            *by_source.entry(example.source.clone()).or_default() += 1;
            if example.verified {
                verified_count += 1;
            }
        }

        DatasetStats {
            total: self.examples.len(),
            verified: verified_count,
            by_error_code,
            by_decision,
            by_source,
        }
    }
}

impl Default for DatasetExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Dataset statistics
#[derive(Debug, Clone)]
pub struct DatasetStats {
    pub total: usize,
    pub verified: usize,
    pub by_error_code: std::collections::HashMap<String, usize>,
    pub by_decision: std::collections::HashMap<String, usize>,
    pub by_source: std::collections::HashMap<String, usize>,
}

impl DatasetStats {
    /// Format as markdown table
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();

        s.push_str("## Dataset Statistics\n\n");
        s.push_str(&format!("- **Total examples**: {}\n", self.total));
        s.push_str(&format!("- **Verified**: {}\n", self.verified));
        s.push_str(&format!(
            "- **Verification rate**: {:.1}%\n\n",
            if self.total > 0 {
                (self.verified as f64 / self.total as f64) * 100.0
            } else {
                0.0
            }
        ));

        s.push_str("### By Error Code\n\n");
        s.push_str("| Error Code | Count |\n");
        s.push_str("|------------|-------|\n");
        let mut codes: Vec<_> = self.by_error_code.iter().collect();
        codes.sort_by_key(|(k, _)| *k);
        for (code, count) in codes {
            s.push_str(&format!("| {} | {} |\n", code, count));
        }

        s.push_str("\n### By Decision Type\n\n");
        s.push_str("| Decision | Count |\n");
        s.push_str("|----------|-------|\n");
        let mut decisions: Vec<_> = self.by_decision.iter().collect();
        decisions.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        for (decision, count) in decisions {
            s.push_str(&format!("| {} | {} |\n", decision, count));
        }

        s.push_str("\n### By Source\n\n");
        s.push_str("| Source | Count |\n");
        s.push_str("|--------|-------|\n");
        let mut sources: Vec<_> = self.by_source.iter().collect();
        sources.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        for (source, count) in sources {
            s.push_str(&format!("| {} | {} |\n", source, count));
        }

        s
    }
}

/// Generate a HuggingFace dataset card (README.md)
pub fn generate_dataset_card(stats: &DatasetStats) -> String {
    format!(
        r#"---
license: mit
task_categories:
  - text2text-generation
language:
  - en
tags:
  - code
  - rust
  - c
  - transpiler
  - compiler-errors
  - code-repair
size_categories:
  - n<1K
---

# Decy Oracle Patterns Dataset

Fix patterns for C→Rust transpilation errors, learned by the decy CITL (Compiler-in-the-Loop Training) oracle.

## Dataset Description

This dataset contains error→fix pairs for common Rust compilation errors that occur during C-to-Rust transpilation.
Each example maps a rustc error code to a fix pattern that resolves the error.

### Use Cases

- **Fine-tuning LLMs** for code repair tasks
- **Training code completion models** for Rust
- **Building retrieval-augmented generation (RAG)** systems for compiler error fixing
- **Studying common C→Rust migration patterns**

## Dataset Structure

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `error_code` | string | Rust compiler error code (e.g., "E0308") |
| `decision` | string | Fix strategy category (e.g., "type_coercion") |
| `fix_diff` | string | Unified diff showing the fix |
| `description` | string | Human-readable explanation |
| `source` | string | Pattern origin (bootstrap, training, imported) |
| `verified` | bool | Whether fix was verified by rustc |
| `success_count` | int | Times pattern succeeded |
| `failure_count` | int | Times pattern failed |

{}

## Usage

### With HuggingFace Datasets

```python
from datasets import load_dataset

dataset = load_dataset("paiml/decy-oracle-patterns")
print(dataset["train"][0])
```

### With alimentar (Rust)

```rust
use alimentar::hf_hub::HfDataset;

let dataset = HfDataset::builder("paiml/decy-oracle-patterns")
    .split("train")
    .build()?
    .download()?;
```

## Error Codes Covered

| Code | Description | Count |
|------|-------------|-------|
| E0308 | Type mismatch | Common in C→Rust type conversions |
| E0133 | Unsafe block required | Raw pointer operations |
| E0382 | Use of moved value | Ownership violations |
| E0499 | Multiple mutable borrows | Borrow checker errors |
| E0506 | Cannot assign to borrowed | Mutation during borrow |
| E0515 | Cannot return reference to local | Lifetime errors |
| E0597 | Value does not live long enough | Lifetime errors |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Citation

```bibtex
@software{{decy2025,
  author       = {{paiml}},
  title        = {{decy: C-to-Rust Transpiler with CITL Oracle}},
  year         = {{2025}},
  publisher    = {{GitHub}},
  url          = {{https://github.com/paiml/decy}}
}}
```

## Related Projects

- [decy](https://github.com/paiml/decy) - C→Rust transpiler
- [depyler](https://github.com/paiml/depyler) - Python→Rust transpiler
- [entrenar](https://github.com/paiml/entrenar) - CITL training framework
- [alimentar](https://github.com/paiml/alimentar) - Data loading library
"#,
        stats.to_markdown()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_exporter_new() {
        let exporter = DatasetExporter::new();
        assert!(!exporter.is_empty());
        assert!(exporter.len() >= 20); // At least 20 bootstrap patterns
    }

    #[test]
    fn test_dataset_exporter_empty() {
        let exporter = DatasetExporter::empty();
        assert!(exporter.is_empty());
        assert_eq!(exporter.len(), 0);
    }

    #[test]
    fn test_add_example() {
        let mut exporter = DatasetExporter::empty();
        exporter.add_example(TrainingExample {
            error_code: "E0308".to_string(),
            decision: "test".to_string(),
            fix_diff: "- old\n+ new".to_string(),
            description: "Test pattern".to_string(),
            source: "test".to_string(),
            verified: true,
            success_count: 0,
            failure_count: 0,
        });
        assert_eq!(exporter.len(), 1);
    }

    #[test]
    fn test_export_jsonl() {
        let exporter = DatasetExporter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("patterns.jsonl");

        let count = exporter.export_jsonl(&path).unwrap();
        assert!(count > 0);
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), count);

        // Verify each line is valid JSON
        for line in lines {
            let _: TrainingExample = serde_json::from_str(line).unwrap();
        }
    }

    #[test]
    fn test_export_chatml() {
        let exporter = DatasetExporter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("patterns_chatml.jsonl");

        let count = exporter.export_chatml(&path).unwrap();
        assert!(count > 0);
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), count);

        // Verify each line is valid ChatML
        for line in lines {
            let conv: ChatMLConversation = serde_json::from_str(line).unwrap();
            assert_eq!(conv.messages.len(), 2);
            assert_eq!(conv.messages[0].role, "user");
            assert_eq!(conv.messages[1].role, "assistant");
        }
    }

    #[test]
    fn test_export_alpaca() {
        let exporter = DatasetExporter::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("patterns_alpaca.jsonl");

        let count = exporter.export_alpaca(&path).unwrap();
        assert!(count > 0);
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), count);

        // Verify each line is valid Alpaca format
        for line in lines {
            let alpaca: AlpacaExample = serde_json::from_str(line).unwrap();
            assert!(!alpaca.instruction.is_empty());
        }
    }

    #[test]
    fn test_stats() {
        let exporter = DatasetExporter::new();
        let stats = exporter.stats();

        assert!(stats.total > 0);
        assert!(!stats.by_error_code.is_empty());
        assert!(!stats.by_decision.is_empty());
        assert!(stats.by_source.contains_key("bootstrap"));
    }

    #[test]
    fn test_stats_markdown() {
        let exporter = DatasetExporter::new();
        let stats = exporter.stats();
        let markdown = stats.to_markdown();

        assert!(markdown.contains("Dataset Statistics"));
        assert!(markdown.contains("Error Code"));
        assert!(markdown.contains("Decision"));
    }

    #[test]
    fn test_generate_dataset_card() {
        let exporter = DatasetExporter::new();
        let stats = exporter.stats();
        let card = generate_dataset_card(&stats);

        assert!(card.contains("license: mit"));
        assert!(card.contains("decy"));
        assert!(card.contains("paiml"));
        assert!(card.contains("Dataset Statistics"));
    }

    #[test]
    fn test_training_example_from_bootstrap() {
        use crate::bootstrap::get_bootstrap_patterns;

        let patterns = get_bootstrap_patterns();
        let pattern = &patterns[0];
        let example = TrainingExample::from_bootstrap(pattern);

        assert_eq!(example.error_code, pattern.error_code);
        assert_eq!(example.decision, pattern.decision);
        assert_eq!(example.source, "bootstrap");
        assert!(example.verified);
    }
}
