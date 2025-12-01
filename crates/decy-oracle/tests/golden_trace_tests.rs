//! Golden Trace Tests - DECY-107
//!
//! RED Phase: These tests define the contract for GoldenTrace struct
//! per unified spec Section 6.3.
//!
//! Tests verify:
//! - GoldenTrace struct creation
//! - TraceTier enum (P0, P1, P2)
//! - SafetyTransformation tracking
//! - JSONL serialization
//! - ChatML format export
//! - Alpaca format export
//! - Dataset statistics

use decy_oracle::golden_trace::{
    GoldenTrace, GoldenTraceDataset, SafetyTransformation, TraceTier, TransformationKind,
};

// ============================================================================
// GOLDEN TRACE CREATION TESTS
// ============================================================================

#[test]
fn test_golden_trace_creation_basic() {
    let trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    assert_eq!(trace.c_snippet, "int x = 10;");
    assert_eq!(trace.rust_snippet, "let x: i32 = 10;");
    assert_eq!(trace.metadata.tier, TraceTier::P0);
    assert_eq!(trace.metadata.source_file, "test.c");
}

#[test]
fn test_golden_trace_has_unique_id() {
    let trace1 = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );
    let trace2 = GoldenTrace::new(
        "int y = 20;".to_string(),
        "let y: i32 = 20;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    assert!(!trace1.metadata.trace_id.is_empty());
    assert!(!trace2.metadata.trace_id.is_empty());
    assert_ne!(trace1.metadata.trace_id, trace2.metadata.trace_id);
}

#[test]
fn test_golden_trace_has_timestamp() {
    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    assert!(!trace.metadata.generated_at.is_empty());
    // Should be ISO 8601 format
    assert!(trace.metadata.generated_at.contains("T"));
}

#[test]
fn test_golden_trace_has_decy_version() {
    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    assert!(!trace.metadata.decy_version.is_empty());
}

// ============================================================================
// TRACE TIER TESTS
// ============================================================================

#[test]
fn test_trace_tier_p0() {
    let tier = TraceTier::P0;
    assert_eq!(tier.to_string(), "P0");
}

#[test]
fn test_trace_tier_p1() {
    let tier = TraceTier::P1;
    assert_eq!(tier.to_string(), "P1");
}

#[test]
fn test_trace_tier_p2() {
    let tier = TraceTier::P2;
    assert_eq!(tier.to_string(), "P2");
}

#[test]
fn test_trace_tier_from_str_valid() {
    assert_eq!("P0".parse::<TraceTier>().unwrap(), TraceTier::P0);
    assert_eq!("p1".parse::<TraceTier>().unwrap(), TraceTier::P1);
    assert_eq!("P2".parse::<TraceTier>().unwrap(), TraceTier::P2);
}

#[test]
fn test_trace_tier_from_str_invalid() {
    assert!("P3".parse::<TraceTier>().is_err());
    assert!("invalid".parse::<TraceTier>().is_err());
    assert!("".parse::<TraceTier>().is_err());
}

// ============================================================================
// SAFETY TRANSFORMATION TESTS
// ============================================================================

#[test]
fn test_add_transformation_malloc_to_box() {
    let mut trace = GoldenTrace::new(
        "int* p = malloc(sizeof(int));".to_string(),
        "let p = Box::new(0i32);".to_string(),
        TraceTier::P1,
        "test.c",
    );

    trace.add_transformation(SafetyTransformation {
        kind: TransformationKind::MallocToBox,
        c_pattern: "malloc(sizeof(int))".to_string(),
        rust_pattern: "Box::new(0i32)".to_string(),
    });

    assert_eq!(trace.metadata.transformations.len(), 1);
    assert!(matches!(
        trace.metadata.transformations[0].kind,
        TransformationKind::MallocToBox
    ));
}

#[test]
fn test_add_multiple_transformations() {
    let mut trace = GoldenTrace::new(
        "int* arr = calloc(10, sizeof(int));".to_string(),
        "let arr: Vec<i32> = vec![0; 10];".to_string(),
        TraceTier::P1,
        "test.c",
    );

    trace.add_transformation(SafetyTransformation {
        kind: TransformationKind::CallocToVec,
        c_pattern: "calloc(10, sizeof(int))".to_string(),
        rust_pattern: "vec![0; 10]".to_string(),
    });

    trace.add_transformation(SafetyTransformation {
        kind: TransformationKind::PointerToReference,
        c_pattern: "int*".to_string(),
        rust_pattern: "Vec<i32>".to_string(),
    });

    assert_eq!(trace.metadata.transformations.len(), 2);
}

// ============================================================================
// SAFETY EXPLANATION (CHAIN OF THOUGHT) TESTS
// ============================================================================

#[test]
fn test_set_explanation() {
    let mut trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    trace.set_explanation("Direct translation of integer declaration.".to_string());

    assert_eq!(
        trace.safety_explanation,
        "Direct translation of integer declaration."
    );
}

#[test]
fn test_generate_explanation_from_transformations() {
    let mut trace = GoldenTrace::new(
        "int* p = malloc(sizeof(int));".to_string(),
        "let p = Box::new(0i32);".to_string(),
        TraceTier::P1,
        "test.c",
    );

    trace.add_transformation(SafetyTransformation {
        kind: TransformationKind::MallocToBox,
        c_pattern: "malloc".to_string(),
        rust_pattern: "Box::new".to_string(),
    });

    trace.generate_explanation();

    assert!(trace.safety_explanation.contains("MallocToBox"));
    assert!(trace.safety_explanation.contains("malloc"));
    assert!(trace.safety_explanation.contains("Box::new"));
}

// ============================================================================
// JSONL SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_to_jsonl_basic() {
    let trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let jsonl = trace.to_jsonl().expect("Failed to serialize");

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&jsonl).expect("Invalid JSON");

    assert_eq!(parsed["c_snippet"], "int x = 10;");
    assert_eq!(parsed["rust_snippet"], "let x: i32 = 10;");
    assert_eq!(parsed["metadata"]["tier"], "P0");
}

#[test]
fn test_jsonl_roundtrip() {
    let original = GoldenTrace::new(
        "int* p = malloc(4);".to_string(),
        "let p = Box::new(0i32);".to_string(),
        TraceTier::P1,
        "test.c",
    );

    let jsonl = original.to_jsonl().expect("Failed to serialize");
    let restored: GoldenTrace = serde_json::from_str(&jsonl).expect("Failed to deserialize");

    assert_eq!(original.c_snippet, restored.c_snippet);
    assert_eq!(original.rust_snippet, restored.rust_snippet);
    assert_eq!(original.metadata.tier, restored.metadata.tier);
}

// ============================================================================
// CHATML FORMAT TESTS
// ============================================================================

#[test]
fn test_to_chatml_format() {
    let mut trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );
    trace.set_explanation("Direct translation.".to_string());

    let chatml = trace.to_chatml();

    // Check required ChatML markers
    assert!(chatml.contains("<|im_start|>system"));
    assert!(chatml.contains("<|im_end|>"));
    assert!(chatml.contains("<|im_start|>user"));
    assert!(chatml.contains("<|im_start|>assistant"));

    // Check content
    assert!(chatml.contains("int x = 10;"));
    assert!(chatml.contains("let x: i32 = 10;"));
    assert!(chatml.contains("Direct translation."));
}

#[test]
fn test_chatml_has_system_prompt() {
    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let chatml = trace.to_chatml();

    // System prompt should mention C-to-Rust
    assert!(chatml.contains("C-to-Rust") || chatml.contains("transpiler"));
}

// ============================================================================
// ALPACA FORMAT TESTS
// ============================================================================

#[test]
fn test_to_alpaca_format() {
    let trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let alpaca = trace.to_alpaca();

    assert!(alpaca["instruction"].as_str().is_some());
    assert_eq!(alpaca["input"], "int x = 10;");
    assert!(alpaca["output"]
        .as_str()
        .unwrap()
        .contains("let x: i32 = 10;"));
}

#[test]
fn test_alpaca_has_metadata() {
    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P1,
        "test.c",
    );

    let alpaca = trace.to_alpaca();

    assert!(alpaca["metadata"].is_object());
    assert_eq!(alpaca["metadata"]["tier"], "P1");
}

// ============================================================================
// DATASET TESTS
// ============================================================================

#[test]
fn test_dataset_creation() {
    let dataset = GoldenTraceDataset::new();

    assert_eq!(dataset.traces.len(), 0);
    assert_eq!(dataset.stats.total_traces, 0);
}

#[test]
fn test_dataset_add_trace() {
    let mut dataset = GoldenTraceDataset::new();

    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    dataset.add_trace(trace);

    assert_eq!(dataset.traces.len(), 1);
    assert_eq!(dataset.stats.total_traces, 1);
}

#[test]
fn test_dataset_stats_by_tier() {
    let mut dataset = GoldenTraceDataset::new();

    dataset.add_trace(GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "a.c",
    ));
    dataset.add_trace(GoldenTrace::new(
        "int* p;".to_string(),
        "let p: *mut i32;".to_string(),
        TraceTier::P1,
        "b.c",
    ));
    dataset.add_trace(GoldenTrace::new(
        "int y;".to_string(),
        "let y: i32;".to_string(),
        TraceTier::P0,
        "c.c",
    ));

    assert_eq!(dataset.stats.traces_by_tier.get("P0"), Some(&2));
    assert_eq!(dataset.stats.traces_by_tier.get("P1"), Some(&1));
}

#[test]
fn test_dataset_avg_lengths() {
    let mut dataset = GoldenTraceDataset::new();

    dataset.add_trace(GoldenTrace::new(
        "int x;".to_string(),      // 6 chars
        "let x: i32;".to_string(), // 11 chars
        TraceTier::P0,
        "test.c",
    ));
    dataset.add_trace(GoldenTrace::new(
        "int y;".to_string(),      // 6 chars
        "let y: i32;".to_string(), // 11 chars
        TraceTier::P0,
        "test.c",
    ));

    assert!((dataset.stats.avg_c_length - 6.0).abs() < 0.01);
    assert!((dataset.stats.avg_rust_length - 11.0).abs() < 0.01);
}

// ============================================================================
// DATASET EXPORT TESTS
// ============================================================================

#[test]
fn test_dataset_generate_card() {
    let mut dataset = GoldenTraceDataset::new();

    dataset.add_trace(GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    ));

    let card = dataset.generate_card();

    assert!(card.contains("Golden Traces Dataset"));
    assert!(card.contains("Total Traces"));
    assert!(card.contains("P0"));
}

#[test]
fn test_dataset_export_jsonl() {
    let mut dataset = GoldenTraceDataset::new();

    dataset.add_trace(GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32;".to_string(),
        TraceTier::P0,
        "test.c",
    ));

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("traces.jsonl");

    dataset.export_jsonl(&output_path).expect("Export failed");

    assert!(output_path.exists());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("int x;"));
    assert!(content.contains("let x: i32;"));
}

// ============================================================================
// CONTEXT TESTS
// ============================================================================

#[test]
fn test_trace_context_headers() {
    let mut trace = GoldenTrace::new(
        "#include <stdio.h>\nint main() {}".to_string(),
        "fn main() {}".to_string(),
        TraceTier::P0,
        "test.c",
    );

    trace.c_context.headers.push("stdio.h".to_string());

    assert_eq!(trace.c_context.headers.len(), 1);
    assert_eq!(trace.c_context.headers[0], "stdio.h");
}

#[test]
fn test_trace_context_type_definitions() {
    let mut trace = GoldenTrace::new(
        "struct Point { int x; int y; };".to_string(),
        "struct Point { x: i32, y: i32 }".to_string(),
        TraceTier::P0,
        "test.c",
    );

    trace
        .c_context
        .type_definitions
        .insert("Point".to_string(), "struct { int x; int y; }".to_string());

    assert!(trace.c_context.type_definitions.contains_key("Point"));
}
