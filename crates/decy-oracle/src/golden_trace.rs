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
            let json = trace.to_jsonl().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;
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
        let json = serde_json::to_string_pretty(&examples).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
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
