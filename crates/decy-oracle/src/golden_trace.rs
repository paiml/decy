//! Golden Trace: Verified C→Rust pairs for model training
//!
//! A Golden Trace represents a high-quality training example where:
//! - C source code is successfully transpiled to Rust
//! - The Rust code compiles without errors
//! - The Rust code is verified as safe (passes decy-verify)
//!
//! These traces form the training dataset for the decy model.
//! Per unified spec Section 6.3.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A verified C→Rust training example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTrace {
    /// Input C source code
    pub c_snippet: String,

    /// Context about the C code (types, headers, etc.)
    pub c_context: TraceContext,

    /// Output Rust code (target for model training)
    pub rust_snippet: String,

    /// Chain-of-thought explanation of the transformation
    pub safety_explanation: String,

    /// Metadata about this trace
    pub metadata: TraceMetadata,
}

/// Context information about the C source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceContext {
    /// Headers included in the C source
    pub headers: Vec<String>,

    /// Type definitions visible in scope
    pub type_definitions: HashMap<String, String>,

    /// Function signatures visible in scope
    pub function_signatures: HashMap<String, String>,

    /// Global variables
    pub globals: Vec<String>,
}

/// Metadata about the training trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Unique identifier for this trace
    pub trace_id: String,

    /// Source file this trace was extracted from
    pub source_file: String,

    /// Complexity tier (P0, P1, P2)
    pub tier: TraceTier,

    /// C99 constructs used in this trace
    pub constructs_used: Vec<String>,

    /// Safety transformations applied
    pub transformations: Vec<SafetyTransformation>,

    /// Whether this trace was human-verified
    pub human_verified: bool,

    /// Timestamp when trace was generated
    pub generated_at: String,

    /// Version of decy that generated this trace
    pub decy_version: String,
}

/// Complexity tier for training curriculum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceTier {
    /// Simple patterns: type mismatches, basic transformations
    P0,
    /// I/O patterns: file handling, format strings
    P1,
    /// Complex patterns: ownership, lifetimes, concurrency
    P2,
}

impl std::fmt::Display for TraceTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceTier::P0 => write!(f, "P0"),
            TraceTier::P1 => write!(f, "P1"),
            TraceTier::P2 => write!(f, "P2"),
        }
    }
}

impl std::str::FromStr for TraceTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "P0" => Ok(TraceTier::P0),
            "P1" => Ok(TraceTier::P1),
            "P2" => Ok(TraceTier::P2),
            _ => Err(format!("Invalid tier: {}. Expected P0, P1, or P2", s)),
        }
    }
}

/// A safety transformation applied during transpilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyTransformation {
    /// Type of transformation
    pub kind: TransformationKind,

    /// Original C pattern
    pub c_pattern: String,

    /// Resulting Rust pattern
    pub rust_pattern: String,
}

/// Types of safety transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationKind {
    /// malloc/free → Box
    MallocToBox,
    /// calloc → Vec
    CallocToVec,
    /// NULL pointer → Option
    NullToOption,
    /// Raw pointer → reference
    PointerToReference,
    /// pthread_mutex → Mutex
    PthreadToMutex,
    /// Output param → Result/return
    OutputParamToResult,
    /// Tagged union → enum
    TaggedUnionToEnum,
    /// Array param → slice
    ArrayParamToSlice,
    /// Other transformation
    Other(String),
}

/// Collection of golden traces for export
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoldenTraceDataset {
    /// All traces in the dataset
    pub traces: Vec<GoldenTrace>,

    /// Dataset statistics
    pub stats: DatasetStats,

    /// Dataset metadata
    pub metadata: DatasetMetadata,
}

/// Statistics about the dataset
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatasetStats {
    /// Total number of traces
    pub total_traces: usize,

    /// Traces by tier
    pub traces_by_tier: HashMap<String, usize>,

    /// Traces by transformation type
    pub traces_by_transformation: HashMap<String, usize>,

    /// Average C snippet length (chars)
    pub avg_c_length: f64,

    /// Average Rust snippet length (chars)
    pub avg_rust_length: f64,
}

/// Metadata about the dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// Dataset version
    pub version: String,

    /// When the dataset was created
    pub created_at: String,

    /// Description of the dataset
    pub description: String,

    /// License
    pub license: String,
}

impl Default for DatasetMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            description: "Golden Traces for C→Rust model training".to_string(),
            license: "Apache-2.0".to_string(),
        }
    }
}

impl GoldenTrace {
    /// Create a new golden trace
    pub fn new(
        c_snippet: String,
        rust_snippet: String,
        tier: TraceTier,
        source_file: &str,
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        c_snippet.hash(&mut hasher);
        rust_snippet.hash(&mut hasher);
        chrono::Utc::now().timestamp_nanos_opt().hash(&mut hasher);
        let trace_id = format!("trace_{:016x}", hasher.finish());

        Self {
            c_snippet,
            c_context: TraceContext::default(),
            rust_snippet,
            safety_explanation: String::new(),
            metadata: TraceMetadata {
                trace_id,
                source_file: source_file.to_string(),
                tier,
                constructs_used: Vec::new(),
                transformations: Vec::new(),
                human_verified: false,
                generated_at: chrono::Utc::now().to_rfc3339(),
                decy_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Add a safety transformation
    pub fn add_transformation(&mut self, transformation: SafetyTransformation) {
        self.metadata.transformations.push(transformation);
    }

    /// Set the safety explanation (chain-of-thought)
    pub fn set_explanation(&mut self, explanation: String) {
        self.safety_explanation = explanation;
    }

    /// Builder-style method to add safety explanation
    pub fn with_safety_explanation(mut self, explanation: &str) -> Self {
        self.safety_explanation = explanation.to_string();
        self
    }

    /// Generate a chain-of-thought explanation based on transformations
    pub fn generate_explanation(&mut self) {
        let mut explanation = String::new();
        explanation.push_str("Safety transformations applied:\n");

        for (i, t) in self.metadata.transformations.iter().enumerate() {
            explanation.push_str(&format!(
                "{}. {:?}: {} → {}\n",
                i + 1,
                t.kind,
                t.c_pattern,
                t.rust_pattern
            ));
        }

        if self.metadata.transformations.is_empty() {
            explanation.push_str("Direct translation with no unsafe patterns detected.\n");
        }

        self.safety_explanation = explanation;
    }

    /// Export to JSONL format (one JSON object per line)
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Export to ChatML format for instruction fine-tuning
    pub fn to_chatml(&self) -> String {
        format!(
            r#"<|im_start|>system
You are a C-to-Rust transpiler that generates safe, idiomatic Rust code.
<|im_end|>
<|im_start|>user
Transpile this C code to safe Rust:
```c
{}
```
<|im_end|>
<|im_start|>assistant
{}
```rust
{}
```
<|im_end|>"#,
            self.c_snippet.trim(),
            self.safety_explanation.trim(),
            self.rust_snippet.trim()
        )
    }

    /// Export to Alpaca format
    pub fn to_alpaca(&self) -> serde_json::Value {
        serde_json::json!({
            "instruction": "Transpile this C code to safe, idiomatic Rust code.",
            "input": self.c_snippet,
            "output": format!("{}\n\n```rust\n{}\n```", self.safety_explanation, self.rust_snippet),
            "metadata": {
                "tier": self.metadata.tier.to_string(),
                "transformations": self.metadata.transformations.len(),
            }
        })
    }
}

impl GoldenTraceDataset {
    /// Create a new empty dataset
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a trace to the dataset
    pub fn add_trace(&mut self, trace: GoldenTrace) {
        self.traces.push(trace);
        self.update_stats();
    }

    /// Update dataset statistics
    fn update_stats(&mut self) {
        self.stats.total_traces = self.traces.len();

        // Count by tier
        self.stats.traces_by_tier.clear();
        for trace in &self.traces {
            *self
                .stats
                .traces_by_tier
                .entry(trace.metadata.tier.to_string())
                .or_insert(0) += 1;
        }

        // Count by transformation
        self.stats.traces_by_transformation.clear();
        for trace in &self.traces {
            for t in &trace.metadata.transformations {
                let key = format!("{:?}", t.kind);
                *self.stats.traces_by_transformation.entry(key).or_insert(0) += 1;
            }
        }

        // Calculate averages
        if !self.traces.is_empty() {
            let total_c: usize = self.traces.iter().map(|t| t.c_snippet.len()).sum();
            let total_rust: usize = self.traces.iter().map(|t| t.rust_snippet.len()).sum();
            self.stats.avg_c_length = total_c as f64 / self.traces.len() as f64;
            self.stats.avg_rust_length = total_rust as f64 / self.traces.len() as f64;
        }
    }

    /// Export entire dataset to JSONL file
    pub fn export_jsonl(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        for trace in &self.traces {
            let json = trace
                .to_jsonl()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
            writeln!(file, "{}", json)?;
        }

        Ok(())
    }

    /// Export entire dataset to ChatML file
    pub fn export_chatml(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        for trace in &self.traces {
            writeln!(file, "{}\n", trace.to_chatml())?;
        }

        Ok(())
    }

    /// Export entire dataset to Alpaca JSON file
    pub fn export_alpaca(&self, path: &Path) -> std::io::Result<()> {
        let examples: Vec<_> = self.traces.iter().map(|t| t.to_alpaca()).collect();
        let json = serde_json::to_string_pretty(&examples)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, json)
    }

    /// Generate dataset card (markdown)
    pub fn generate_card(&self) -> String {
        format!(
            r#"# Golden Traces Dataset

## Overview

This dataset contains verified C→Rust translation pairs for training
C-to-Rust transpilation models.

## Statistics

- **Total Traces**: {}
- **P0 (Simple)**: {}
- **P1 (I/O)**: {}
- **P2 (Complex)**: {}
- **Avg C Length**: {:.0} chars
- **Avg Rust Length**: {:.0} chars

## Transformations

{}

## Usage

```python
from datasets import load_dataset
dataset = load_dataset("paiml/decy-golden-traces")
```

## License

{}

## Citation

```bibtex
@software{{decy_golden_traces,
  title = {{DECY Golden Traces Dataset}},
  year = {{2025}},
  publisher = {{PAIML}},
}}
```
"#,
            self.stats.total_traces,
            self.stats.traces_by_tier.get("P0").unwrap_or(&0),
            self.stats.traces_by_tier.get("P1").unwrap_or(&0),
            self.stats.traces_by_tier.get("P2").unwrap_or(&0),
            self.stats.avg_c_length,
            self.stats.avg_rust_length,
            self.stats
                .traces_by_transformation
                .iter()
                .map(|(k, v)| format!("- **{}**: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n"),
            self.metadata.license,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trace(tier: TraceTier) -> GoldenTrace {
        GoldenTrace::new(
            "int x = 42;".to_string(),
            "let x: i32 = 42;".to_string(),
            tier,
            "test.c",
        )
    }

    // ========================================================================
    // GoldenTrace creation and builders
    // ========================================================================

    #[test]
    fn test_golden_trace_new() {
        let trace = sample_trace(TraceTier::P0);
        assert_eq!(trace.c_snippet, "int x = 42;");
        assert_eq!(trace.rust_snippet, "let x: i32 = 42;");
        assert_eq!(trace.metadata.tier, TraceTier::P0);
        assert_eq!(trace.metadata.source_file, "test.c");
        assert!(trace.metadata.trace_id.starts_with("trace_"));
        assert!(!trace.metadata.human_verified);
    }

    #[test]
    fn test_golden_trace_with_safety_explanation() {
        let trace = sample_trace(TraceTier::P0)
            .with_safety_explanation("Direct integer mapping, no unsafe required");
        assert_eq!(
            trace.safety_explanation,
            "Direct integer mapping, no unsafe required"
        );
    }

    #[test]
    fn test_golden_trace_set_explanation() {
        let mut trace = sample_trace(TraceTier::P0);
        trace.set_explanation("test explanation".to_string());
        assert_eq!(trace.safety_explanation, "test explanation");
    }

    #[test]
    fn test_golden_trace_add_transformation() {
        let mut trace = sample_trace(TraceTier::P1);
        trace.add_transformation(SafetyTransformation {
            kind: TransformationKind::MallocToBox,
            c_pattern: "malloc(sizeof(int))".to_string(),
            rust_pattern: "Box::new(0i32)".to_string(),
        });
        assert_eq!(trace.metadata.transformations.len(), 1);
    }

    // ========================================================================
    // generate_explanation
    // ========================================================================

    #[test]
    fn test_generate_explanation_with_transformations() {
        let mut trace = sample_trace(TraceTier::P2);
        trace.add_transformation(SafetyTransformation {
            kind: TransformationKind::MallocToBox,
            c_pattern: "malloc()".to_string(),
            rust_pattern: "Box::new()".to_string(),
        });
        trace.add_transformation(SafetyTransformation {
            kind: TransformationKind::NullToOption,
            c_pattern: "NULL".to_string(),
            rust_pattern: "None".to_string(),
        });
        trace.generate_explanation();
        assert!(trace.safety_explanation.contains("Safety transformations"));
        assert!(trace.safety_explanation.contains("MallocToBox"));
        assert!(trace.safety_explanation.contains("NullToOption"));
    }

    #[test]
    fn test_generate_explanation_no_transformations() {
        let mut trace = sample_trace(TraceTier::P0);
        trace.generate_explanation();
        assert!(trace
            .safety_explanation
            .contains("Direct translation with no unsafe patterns detected"));
    }

    // ========================================================================
    // Export formats: JSONL, ChatML, Alpaca
    // ========================================================================

    #[test]
    fn test_to_jsonl() {
        let trace = sample_trace(TraceTier::P0)
            .with_safety_explanation("Safe mapping");
        let jsonl = trace.to_jsonl().unwrap();
        assert!(jsonl.contains("int x = 42;"));
        assert!(jsonl.contains("let x: i32 = 42;"));
        // Verify it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&jsonl).unwrap();
    }

    #[test]
    fn test_to_chatml() {
        let trace = sample_trace(TraceTier::P0)
            .with_safety_explanation("Direct mapping");
        let chatml = trace.to_chatml();
        assert!(chatml.contains("<|im_start|>system"));
        assert!(chatml.contains("<|im_start|>user"));
        assert!(chatml.contains("<|im_start|>assistant"));
        assert!(chatml.contains("int x = 42;"));
        assert!(chatml.contains("let x: i32 = 42;"));
        assert!(chatml.contains("Direct mapping"));
    }

    #[test]
    fn test_to_alpaca() {
        let trace = sample_trace(TraceTier::P1)
            .with_safety_explanation("Transformed pattern");
        let alpaca = trace.to_alpaca();
        assert_eq!(
            alpaca["instruction"],
            "Transpile this C code to safe, idiomatic Rust code."
        );
        assert!(alpaca["input"].as_str().unwrap().contains("int x = 42;"));
        assert!(alpaca["output"].as_str().unwrap().contains("let x: i32 = 42;"));
        assert_eq!(alpaca["metadata"]["tier"], "P1");
    }

    // ========================================================================
    // TraceTier Display and FromStr
    // ========================================================================

    #[test]
    fn test_trace_tier_display() {
        assert_eq!(TraceTier::P0.to_string(), "P0");
        assert_eq!(TraceTier::P1.to_string(), "P1");
        assert_eq!(TraceTier::P2.to_string(), "P2");
    }

    #[test]
    fn test_trace_tier_from_str() {
        assert_eq!("P0".parse::<TraceTier>().unwrap(), TraceTier::P0);
        assert_eq!("P1".parse::<TraceTier>().unwrap(), TraceTier::P1);
        assert_eq!("P2".parse::<TraceTier>().unwrap(), TraceTier::P2);
        // Case insensitive
        assert_eq!("p0".parse::<TraceTier>().unwrap(), TraceTier::P0);
    }

    #[test]
    fn test_trace_tier_from_str_invalid() {
        let err = "P3".parse::<TraceTier>().unwrap_err();
        assert!(err.contains("Invalid tier"));
    }

    // ========================================================================
    // TransformationKind all variants
    // ========================================================================

    #[test]
    fn test_all_transformation_kinds_serializable() {
        let kinds = vec![
            TransformationKind::MallocToBox,
            TransformationKind::CallocToVec,
            TransformationKind::NullToOption,
            TransformationKind::PointerToReference,
            TransformationKind::PthreadToMutex,
            TransformationKind::OutputParamToResult,
            TransformationKind::TaggedUnionToEnum,
            TransformationKind::ArrayParamToSlice,
            TransformationKind::Other("custom".to_string()),
        ];

        for kind in kinds {
            let t = SafetyTransformation {
                kind,
                c_pattern: "c".to_string(),
                rust_pattern: "rust".to_string(),
            };
            let json = serde_json::to_string(&t).unwrap();
            let _: SafetyTransformation = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // GoldenTraceDataset
    // ========================================================================

    #[test]
    fn test_dataset_new() {
        let ds = GoldenTraceDataset::new();
        assert_eq!(ds.traces.len(), 0);
        assert_eq!(ds.stats.total_traces, 0);
    }

    #[test]
    fn test_dataset_add_trace() {
        let mut ds = GoldenTraceDataset::new();
        ds.add_trace(sample_trace(TraceTier::P0));
        ds.add_trace(sample_trace(TraceTier::P1));
        ds.add_trace(sample_trace(TraceTier::P2));

        assert_eq!(ds.stats.total_traces, 3);
        assert_eq!(ds.stats.traces_by_tier.get("P0"), Some(&1));
        assert_eq!(ds.stats.traces_by_tier.get("P1"), Some(&1));
        assert_eq!(ds.stats.traces_by_tier.get("P2"), Some(&1));
        assert!(ds.stats.avg_c_length > 0.0);
        assert!(ds.stats.avg_rust_length > 0.0);
    }

    #[test]
    fn test_dataset_add_trace_with_transformations() {
        let mut ds = GoldenTraceDataset::new();
        let mut trace = sample_trace(TraceTier::P2);
        trace.add_transformation(SafetyTransformation {
            kind: TransformationKind::MallocToBox,
            c_pattern: "malloc".to_string(),
            rust_pattern: "Box::new".to_string(),
        });
        ds.add_trace(trace);

        assert_eq!(ds.stats.traces_by_transformation.len(), 1);
    }

    // ========================================================================
    // Dataset exports
    // ========================================================================

    #[test]
    fn test_dataset_export_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");

        let mut ds = GoldenTraceDataset::new();
        ds.add_trace(
            sample_trace(TraceTier::P0).with_safety_explanation("safe"),
        );
        ds.add_trace(
            sample_trace(TraceTier::P1).with_safety_explanation("safe"),
        );

        ds.export_jsonl(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line is valid JSON
        for line in lines {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }
    }

    #[test]
    fn test_dataset_export_chatml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.chatml");

        let mut ds = GoldenTraceDataset::new();
        ds.add_trace(
            sample_trace(TraceTier::P0).with_safety_explanation("safe"),
        );

        ds.export_chatml(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<|im_start|>"));
    }

    #[test]
    fn test_dataset_export_alpaca() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.json");

        let mut ds = GoldenTraceDataset::new();
        ds.add_trace(
            sample_trace(TraceTier::P0).with_safety_explanation("safe"),
        );

        ds.export_alpaca(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let arr: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(arr.len(), 1);
    }

    // ========================================================================
    // Dataset card
    // ========================================================================

    #[test]
    fn test_dataset_generate_card() {
        let mut ds = GoldenTraceDataset::new();
        let mut trace = sample_trace(TraceTier::P0);
        trace.add_transformation(SafetyTransformation {
            kind: TransformationKind::MallocToBox,
            c_pattern: "malloc".to_string(),
            rust_pattern: "Box::new".to_string(),
        });
        ds.add_trace(trace);
        ds.add_trace(sample_trace(TraceTier::P1));

        let card = ds.generate_card();
        assert!(card.contains("Golden Traces Dataset"));
        assert!(card.contains("Total Traces"));
        assert!(card.contains("Apache-2.0"));
    }

    #[test]
    fn test_dataset_generate_card_empty() {
        let ds = GoldenTraceDataset::new();
        let card = ds.generate_card();
        assert!(card.contains("Total Traces"));
    }

    // ========================================================================
    // DatasetMetadata default
    // ========================================================================

    #[test]
    fn test_dataset_metadata_default() {
        let meta = DatasetMetadata::default();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.license, "Apache-2.0");
        assert!(meta.description.contains("Golden Traces"));
    }

    // ========================================================================
    // TraceContext default
    // ========================================================================

    #[test]
    fn test_trace_context_default() {
        let ctx = TraceContext::default();
        assert!(ctx.headers.is_empty());
        assert!(ctx.type_definitions.is_empty());
        assert!(ctx.function_signatures.is_empty());
        assert!(ctx.globals.is_empty());
    }

    // ========================================================================
    // Serialization round-trips
    // ========================================================================

    #[test]
    fn test_golden_trace_serialize_roundtrip() {
        let trace = sample_trace(TraceTier::P0)
            .with_safety_explanation("test");
        let json = serde_json::to_string(&trace).unwrap();
        let deserialized: GoldenTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.c_snippet, trace.c_snippet);
        assert_eq!(deserialized.rust_snippet, trace.rust_snippet);
    }

    #[test]
    fn test_dataset_serialize_roundtrip() {
        let mut ds = GoldenTraceDataset::new();
        ds.add_trace(sample_trace(TraceTier::P0));
        let json = serde_json::to_string(&ds).unwrap();
        let deserialized: GoldenTraceDataset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.traces.len(), 1);
    }
}
