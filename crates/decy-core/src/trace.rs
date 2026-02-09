//! DECY-193: Decision tracing / flight recorder for transpilation.
//!
//! Provides a JSON audit trail of all decisions made during transpilation,
//! including ownership inference, type mapping, and code generation choices.
//!
//! # Examples
//!
//! ```
//! use decy_core::trace::{TraceCollector, TraceEntry, DecisionType, PipelineStage};
//!
//! let mut collector = TraceCollector::new();
//! collector.record(TraceEntry {
//!     stage: PipelineStage::OwnershipInference,
//!     source_location: Some("line 10".to_string()),
//!     decision_type: DecisionType::PointerClassification,
//!     chosen: "Box<i32>".to_string(),
//!     alternatives: vec!["&i32".to_string(), "&mut i32".to_string()],
//!     confidence: 0.95,
//!     reason: "malloc/free pattern detected".to_string(),
//! });
//!
//! assert_eq!(collector.entries().len(), 1);
//! ```

use serde::{Deserialize, Serialize};

/// Stage of the transpilation pipeline where a decision was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    /// C parsing stage
    Parsing,
    /// HIR conversion stage
    HirConversion,
    /// Ownership inference stage
    OwnershipInference,
    /// Lifetime analysis stage
    LifetimeAnalysis,
    /// Code generation stage
    CodeGeneration,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStage::Parsing => write!(f, "parsing"),
            PipelineStage::HirConversion => write!(f, "hir_conversion"),
            PipelineStage::OwnershipInference => write!(f, "ownership_inference"),
            PipelineStage::LifetimeAnalysis => write!(f, "lifetime_analysis"),
            PipelineStage::CodeGeneration => write!(f, "code_generation"),
        }
    }
}

/// Type of decision being recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// Classification of a pointer as owning, borrowing, etc.
    PointerClassification,
    /// Mapping a C type to a Rust type
    TypeMapping,
    /// Choosing a safe pattern over an unsafe one
    SafetyTransformation,
    /// Lifetime annotation decision
    LifetimeAnnotation,
    /// Pattern detection (malloc/free, array, etc.)
    PatternDetection,
    /// Function signature transformation
    SignatureTransformation,
}

impl std::fmt::Display for DecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionType::PointerClassification => write!(f, "pointer_classification"),
            DecisionType::TypeMapping => write!(f, "type_mapping"),
            DecisionType::SafetyTransformation => write!(f, "safety_transformation"),
            DecisionType::LifetimeAnnotation => write!(f, "lifetime_annotation"),
            DecisionType::PatternDetection => write!(f, "pattern_detection"),
            DecisionType::SignatureTransformation => write!(f, "signature_transformation"),
        }
    }
}

/// A single decision recorded during transpilation.
///
/// Each entry captures what was decided, what alternatives existed,
/// and why the chosen option was selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Which pipeline stage made this decision
    pub stage: PipelineStage,
    /// Source location in the C code (e.g., "line 10, column 5")
    pub source_location: Option<String>,
    /// What type of decision was made
    pub decision_type: DecisionType,
    /// The option that was chosen
    pub chosen: String,
    /// Alternative options that were considered but not chosen
    pub alternatives: Vec<String>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Human-readable reason for the decision
    pub reason: String,
}

/// Collects trace entries during transpilation.
///
/// Thread-safe collector that can be passed through the pipeline stages.
///
/// # Examples
///
/// ```
/// use decy_core::trace::{TraceCollector, TraceEntry, DecisionType, PipelineStage};
///
/// let mut collector = TraceCollector::new();
/// assert!(collector.is_empty());
///
/// collector.record(TraceEntry {
///     stage: PipelineStage::CodeGeneration,
///     source_location: None,
///     decision_type: DecisionType::TypeMapping,
///     chosen: "i32".to_string(),
///     alternatives: vec!["i64".to_string()],
///     confidence: 1.0,
///     reason: "C int maps to Rust i32".to_string(),
/// });
///
/// assert_eq!(collector.len(), 1);
/// let json = collector.to_json();
/// assert!(json.contains("i32"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TraceCollector {
    entries: Vec<TraceEntry>,
}

impl TraceCollector {
    /// Create a new empty trace collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a trace entry.
    pub fn record(&mut self, entry: TraceEntry) {
        self.entries.push(entry);
    }

    /// Get all recorded entries.
    pub fn entries(&self) -> &[TraceEntry] {
        &self.entries
    }

    /// Get the number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the collector has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize all entries to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_else(|_| "[]".to_string())
    }

    /// Filter entries by pipeline stage.
    pub fn entries_for_stage(&self, stage: &PipelineStage) -> Vec<&TraceEntry> {
        self.entries.iter().filter(|e| &e.stage == stage).collect()
    }

    /// Get summary statistics.
    pub fn summary(&self) -> TraceSummary {
        let mut decisions_by_stage = std::collections::HashMap::new();
        let mut total_confidence = 0.0;

        for entry in &self.entries {
            *decisions_by_stage
                .entry(entry.stage.to_string())
                .or_insert(0u64) += 1;
            total_confidence += entry.confidence;
        }

        TraceSummary {
            total_decisions: self.entries.len(),
            avg_confidence: if self.entries.is_empty() {
                0.0
            } else {
                total_confidence / self.entries.len() as f64
            },
            decisions_by_stage,
        }
    }
}

/// Summary statistics for a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    /// Total number of decisions made
    pub total_decisions: usize,
    /// Average confidence across all decisions
    pub avg_confidence: f64,
    /// Number of decisions per pipeline stage
    pub decisions_by_stage: std::collections::HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_collector_new_is_empty() {
        let collector = TraceCollector::new();
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_trace_collector_record_entry() {
        let mut collector = TraceCollector::new();
        collector.record(TraceEntry {
            stage: PipelineStage::OwnershipInference,
            source_location: Some("line 5".to_string()),
            decision_type: DecisionType::PointerClassification,
            chosen: "Box<i32>".to_string(),
            alternatives: vec!["&i32".to_string()],
            confidence: 0.9,
            reason: "malloc detected".to_string(),
        });

        assert_eq!(collector.len(), 1);
        assert!(!collector.is_empty());
        assert_eq!(collector.entries()[0].chosen, "Box<i32>");
    }

    #[test]
    fn test_trace_collector_to_json() {
        let mut collector = TraceCollector::new();
        collector.record(TraceEntry {
            stage: PipelineStage::CodeGeneration,
            source_location: None,
            decision_type: DecisionType::TypeMapping,
            chosen: "i32".to_string(),
            alternatives: vec![],
            confidence: 1.0,
            reason: "direct mapping".to_string(),
        });

        let json = collector.to_json();
        assert!(json.contains("i32"));
        assert!(json.contains("code_generation"));
    }

    #[test]
    fn test_trace_collector_filter_by_stage() {
        let mut collector = TraceCollector::new();
        collector.record(TraceEntry {
            stage: PipelineStage::Parsing,
            source_location: None,
            decision_type: DecisionType::TypeMapping,
            chosen: "int".to_string(),
            alternatives: vec![],
            confidence: 1.0,
            reason: "parsed".to_string(),
        });
        collector.record(TraceEntry {
            stage: PipelineStage::OwnershipInference,
            source_location: None,
            decision_type: DecisionType::PointerClassification,
            chosen: "&i32".to_string(),
            alternatives: vec![],
            confidence: 0.8,
            reason: "read-only".to_string(),
        });

        let parsing = collector.entries_for_stage(&PipelineStage::Parsing);
        assert_eq!(parsing.len(), 1);

        let ownership = collector.entries_for_stage(&PipelineStage::OwnershipInference);
        assert_eq!(ownership.len(), 1);
    }

    #[test]
    fn test_trace_summary() {
        let mut collector = TraceCollector::new();
        collector.record(TraceEntry {
            stage: PipelineStage::OwnershipInference,
            source_location: None,
            decision_type: DecisionType::PointerClassification,
            chosen: "Box<i32>".to_string(),
            alternatives: vec![],
            confidence: 0.8,
            reason: "test".to_string(),
        });
        collector.record(TraceEntry {
            stage: PipelineStage::OwnershipInference,
            source_location: None,
            decision_type: DecisionType::PointerClassification,
            chosen: "&i32".to_string(),
            alternatives: vec![],
            confidence: 1.0,
            reason: "test".to_string(),
        });

        let summary = collector.summary();
        assert_eq!(summary.total_decisions, 2);
        assert!((summary.avg_confidence - 0.9).abs() < 0.001);
        assert_eq!(
            summary.decisions_by_stage.get("ownership_inference"),
            Some(&2)
        );
    }
}
