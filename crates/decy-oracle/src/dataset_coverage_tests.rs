//! Coverage expansion tests for dataset.rs
//!
//! Targets uncovered branches: error paths, edge cases, parquet stub,
//! empty stats, serialization round-trips, and Default impl.

use crate::dataset::*;
use std::collections::HashMap;

// ============================================================================
// DatasetExporter: Default trait impl
// ============================================================================

#[test]
fn test_dataset_exporter_default_trait() {
    let exporter = DatasetExporter::default();
    assert!(!exporter.is_empty());
    assert!(exporter.len() >= 20);
}

// ============================================================================
// DatasetExporter: examples() accessor
// ============================================================================

#[test]
fn test_dataset_exporter_examples_accessor() {
    let mut exporter = DatasetExporter::empty();
    assert!(exporter.examples().is_empty());

    exporter.add_example(TrainingExample {
        error_code: "E0308".to_string(),
        decision: "type_coercion".to_string(),
        fix_diff: "- old\n+ new".to_string(),
        description: "Test".to_string(),
        source: "test".to_string(),
        verified: false,
        success_count: 5,
        failure_count: 2,
    });

    assert_eq!(exporter.examples().len(), 1);
    assert_eq!(exporter.examples()[0].error_code, "E0308");
    assert!(!exporter.examples()[0].verified);
    assert_eq!(exporter.examples()[0].success_count, 5);
    assert_eq!(exporter.examples()[0].failure_count, 2);
}

// ============================================================================
// Parquet export: non-dataset feature stub
// ============================================================================

#[test]
fn test_export_parquet_stub_returns_error() {
    let exporter = DatasetExporter::empty();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("patterns.parquet");

    let result = exporter.export_parquet(&path);
    // Without the "dataset" feature, this should return an ExportError
    // With the "dataset" feature, it would succeed on empty data
    // Either way, we exercise the code path
    match result {
        Ok(count) => assert_eq!(count, 0), // dataset feature enabled, empty exporter
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("dataset"));
        }
    }
}

#[test]
fn test_export_parquet_stub_with_data() {
    let exporter = DatasetExporter::new();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("patterns_data.parquet");

    let result = exporter.export_parquet(&path);
    match result {
        Ok(count) => assert!(count > 0),
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("dataset") || msg.contains("feature"));
        }
    }
}

// ============================================================================
// Export error paths: invalid paths cause write failures
// ============================================================================

#[test]
fn test_export_jsonl_write_error() {
    let exporter = DatasetExporter::new();
    // Writing to a directory that doesn't exist should fail
    let path = "/nonexistent_dir_xyz_12345/patterns.jsonl";
    let result = exporter.export_jsonl(path);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to write JSONL"));
    assert!(err_msg.contains("nonexistent_dir_xyz_12345"));
}

#[test]
fn test_export_chatml_write_error() {
    let exporter = DatasetExporter::new();
    let path = "/nonexistent_dir_xyz_12345/patterns_chatml.jsonl";
    let result = exporter.export_chatml(path);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to write ChatML"));
}

#[test]
fn test_export_alpaca_write_error() {
    let exporter = DatasetExporter::new();
    let path = "/nonexistent_dir_xyz_12345/patterns_alpaca.jsonl";
    let result = exporter.export_alpaca(path);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to write Alpaca"));
}

// ============================================================================
// Export with empty exporter: exercises zero-iteration loops
// ============================================================================

#[test]
fn test_export_jsonl_empty() {
    let exporter = DatasetExporter::empty();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("empty.jsonl");

    let count = exporter.export_jsonl(&path).unwrap();
    assert_eq!(count, 0);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.is_empty());
}

#[test]
fn test_export_chatml_empty() {
    let exporter = DatasetExporter::empty();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("empty_chatml.jsonl");

    let count = exporter.export_chatml(&path).unwrap();
    assert_eq!(count, 0);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.is_empty());
}

#[test]
fn test_export_alpaca_empty() {
    let exporter = DatasetExporter::empty();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("empty_alpaca.jsonl");

    let count = exporter.export_alpaca(&path).unwrap();
    assert_eq!(count, 0);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.is_empty());
}

// ============================================================================
// DatasetStats: empty stats edge case
// ============================================================================

#[test]
fn test_stats_empty_exporter() {
    let exporter = DatasetExporter::empty();
    let stats = exporter.stats();

    assert_eq!(stats.total, 0);
    assert_eq!(stats.verified, 0);
    assert!(stats.by_error_code.is_empty());
    assert!(stats.by_decision.is_empty());
    assert!(stats.by_source.is_empty());
}

// ============================================================================
// DatasetStats::to_markdown: empty and populated edge cases
// ============================================================================

#[test]
fn test_stats_markdown_empty() {
    let stats = DatasetStats {
        total: 0,
        verified: 0,
        by_error_code: HashMap::new(),
        by_decision: HashMap::new(),
        by_source: HashMap::new(),
    };
    let md = stats.to_markdown();

    assert!(md.contains("Dataset Statistics"));
    assert!(md.contains("Total examples**: 0"));
    assert!(md.contains("Verified**: 0"));
    // Zero total should produce 0.0% verification rate
    assert!(md.contains("0.0%"));
}

#[test]
fn test_stats_markdown_multiple_entries() {
    let mut by_error_code = HashMap::new();
    by_error_code.insert("E0308".to_string(), 10);
    by_error_code.insert("E0133".to_string(), 5);
    by_error_code.insert("E0382".to_string(), 3);

    let mut by_decision = HashMap::new();
    by_decision.insert("type_coercion".to_string(), 8);
    by_decision.insert("unsafe_block".to_string(), 7);
    by_decision.insert("pointer_to_reference".to_string(), 3);

    let mut by_source = HashMap::new();
    by_source.insert("bootstrap".to_string(), 12);
    by_source.insert("training".to_string(), 6);

    let stats = DatasetStats {
        total: 18,
        verified: 15,
        by_error_code,
        by_decision,
        by_source,
    };

    let md = stats.to_markdown();

    // Check structure
    assert!(md.contains("By Error Code"));
    assert!(md.contains("By Decision Type"));
    assert!(md.contains("By Source"));

    // Error codes sorted alphabetically
    assert!(md.contains("E0133"));
    assert!(md.contains("E0308"));
    assert!(md.contains("E0382"));

    // Decision types present
    assert!(md.contains("type_coercion"));
    assert!(md.contains("unsafe_block"));

    // Sources present
    assert!(md.contains("bootstrap"));
    assert!(md.contains("training"));

    // Verification rate = 15/18 * 100 = 83.3%
    assert!(md.contains("83.3%"));
}

#[test]
fn test_stats_markdown_full_verification() {
    let mut by_error_code = HashMap::new();
    by_error_code.insert("E0308".to_string(), 5);

    let mut by_decision = HashMap::new();
    by_decision.insert("type_coercion".to_string(), 5);

    let mut by_source = HashMap::new();
    by_source.insert("bootstrap".to_string(), 5);

    let stats = DatasetStats {
        total: 5,
        verified: 5,
        by_error_code,
        by_decision,
        by_source,
    };

    let md = stats.to_markdown();
    assert!(md.contains("100.0%"));
}

// ============================================================================
// generate_dataset_card: content verification
// ============================================================================

#[test]
fn test_generate_dataset_card_structure() {
    let stats = DatasetStats {
        total: 0,
        verified: 0,
        by_error_code: HashMap::new(),
        by_decision: HashMap::new(),
        by_source: HashMap::new(),
    };

    let card = generate_dataset_card(&stats);

    // YAML frontmatter
    assert!(card.contains("license: mit"));
    assert!(card.contains("task_categories:"));
    assert!(card.contains("text2text-generation"));
    assert!(card.contains("tags:"));
    assert!(card.contains("- code"));
    assert!(card.contains("- rust"));
    assert!(card.contains("- c"));
    assert!(card.contains("- transpiler"));
    assert!(card.contains("- compiler-errors"));
    assert!(card.contains("- code-repair"));
    assert!(card.contains("size_categories:"));

    // Content sections
    assert!(card.contains("Decy Oracle Patterns Dataset"));
    assert!(card.contains("Dataset Description"));
    assert!(card.contains("Use Cases"));
    assert!(card.contains("Fine-tuning LLMs"));
    assert!(card.contains("Dataset Structure"));
    assert!(card.contains("Fields"));
    assert!(card.contains("error_code"));
    assert!(card.contains("decision"));
    assert!(card.contains("fix_diff"));
    assert!(card.contains("source"));
    assert!(card.contains("verified"));
    assert!(card.contains("success_count"));
    assert!(card.contains("failure_count"));

    // Usage examples
    assert!(card.contains("HuggingFace Datasets"));
    assert!(card.contains("load_dataset"));
    assert!(card.contains("alimentar"));

    // Error codes table
    assert!(card.contains("E0308"));
    assert!(card.contains("E0133"));
    assert!(card.contains("E0382"));

    // License and citation
    assert!(card.contains("MIT License"));
    assert!(card.contains("@software"));
    assert!(card.contains("decy2025"));

    // Related projects
    assert!(card.contains("depyler"));
    assert!(card.contains("entrenar"));
    assert!(card.contains("alimentar"));
}

#[test]
fn test_generate_dataset_card_includes_stats() {
    let mut by_error_code = HashMap::new();
    by_error_code.insert("E0308".to_string(), 10);

    let mut by_decision = HashMap::new();
    by_decision.insert("type_coercion".to_string(), 10);

    let mut by_source = HashMap::new();
    by_source.insert("bootstrap".to_string(), 10);

    let stats = DatasetStats {
        total: 10,
        verified: 8,
        by_error_code,
        by_decision,
        by_source,
    };

    let card = generate_dataset_card(&stats);
    // The card should embed the to_markdown() output
    assert!(card.contains("Dataset Statistics"));
    assert!(card.contains("Total examples**: 10"));
    assert!(card.contains("Verified**: 8"));
}

// ============================================================================
// TrainingExample: serialization round-trip
// ============================================================================

#[test]
fn test_training_example_serde_roundtrip() {
    let example = TrainingExample {
        error_code: "E0308".to_string(),
        decision: "type_coercion".to_string(),
        fix_diff: "- let x: i32 = v;\n+ let x: i32 = v as i32;".to_string(),
        description: "Add explicit cast".to_string(),
        source: "training".to_string(),
        verified: false,
        success_count: 42,
        failure_count: 3,
    };

    let json = serde_json::to_string(&example).unwrap();
    let deserialized: TrainingExample = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.error_code, "E0308");
    assert_eq!(deserialized.decision, "type_coercion");
    assert_eq!(deserialized.source, "training");
    assert!(!deserialized.verified);
    assert_eq!(deserialized.success_count, 42);
    assert_eq!(deserialized.failure_count, 3);
}

#[test]
fn test_training_example_debug() {
    let example = TrainingExample {
        error_code: "E0133".to_string(),
        decision: "unsafe_block".to_string(),
        fix_diff: "- *ptr\n+ unsafe { *ptr }".to_string(),
        description: "Wrap in unsafe".to_string(),
        source: "bootstrap".to_string(),
        verified: true,
        success_count: 0,
        failure_count: 0,
    };
    let debug = format!("{:?}", example);
    assert!(debug.contains("E0133"));
    assert!(debug.contains("unsafe_block"));
}

#[test]
fn test_training_example_clone() {
    let example = TrainingExample {
        error_code: "E0382".to_string(),
        decision: "add_clone".to_string(),
        fix_diff: "- use val\n+ use val.clone()".to_string(),
        description: "Clone to avoid move".to_string(),
        source: "imported".to_string(),
        verified: true,
        success_count: 10,
        failure_count: 1,
    };

    let cloned = example.clone();
    assert_eq!(cloned.error_code, example.error_code);
    assert_eq!(cloned.success_count, example.success_count);
}

// ============================================================================
// ChatMLMessage and ChatMLConversation: serialization
// ============================================================================

#[test]
fn test_chatml_message_serde() {
    let msg = ChatMLMessage {
        role: "user".to_string(),
        content: "Fix this error".to_string(),
    };

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ChatMLMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.role, "user");
    assert_eq!(deserialized.content, "Fix this error");
}

#[test]
fn test_chatml_conversation_serde() {
    let conv = ChatMLConversation {
        messages: vec![
            ChatMLMessage {
                role: "user".to_string(),
                content: "Help me fix E0308".to_string(),
            },
            ChatMLMessage {
                role: "assistant".to_string(),
                content: "Add `as i32` cast".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&conv).unwrap();
    let deserialized: ChatMLConversation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.messages.len(), 2);
    assert_eq!(deserialized.messages[0].role, "user");
    assert_eq!(deserialized.messages[1].role, "assistant");
}

#[test]
fn test_chatml_message_debug_clone() {
    let msg = ChatMLMessage {
        role: "system".to_string(),
        content: "You are a Rust expert.".to_string(),
    };
    let debug = format!("{:?}", msg);
    assert!(debug.contains("system"));

    let cloned = msg.clone();
    assert_eq!(cloned.role, "system");
}

// ============================================================================
// AlpacaExample: serialization
// ============================================================================

#[test]
fn test_alpaca_example_serde() {
    let alpaca = AlpacaExample {
        instruction: "Fix type error".to_string(),
        input: "let x: i32 = val;".to_string(),
        output: "let x: i32 = val as i32;".to_string(),
    };

    let json = serde_json::to_string(&alpaca).unwrap();
    let deserialized: AlpacaExample = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.instruction, "Fix type error");
    assert_eq!(deserialized.input, "let x: i32 = val;");
    assert_eq!(deserialized.output, "let x: i32 = val as i32;");
}

#[test]
fn test_alpaca_example_debug_clone() {
    let alpaca = AlpacaExample {
        instruction: "Fix it".to_string(),
        input: "old".to_string(),
        output: "new".to_string(),
    };
    let debug = format!("{:?}", alpaca);
    assert!(debug.contains("Fix it"));

    let cloned = alpaca.clone();
    assert_eq!(cloned.instruction, "Fix it");
}

// ============================================================================
// ExportFormat: debug, clone, equality
// ============================================================================

#[test]
fn test_export_format_debug() {
    let fmt = ExportFormat::Jsonl;
    let debug = format!("{:?}", fmt);
    assert!(debug.contains("Jsonl"));
}

#[test]
fn test_export_format_eq() {
    assert_eq!(ExportFormat::Jsonl, ExportFormat::Jsonl);
    assert_eq!(ExportFormat::ChatML, ExportFormat::ChatML);
    assert_eq!(ExportFormat::Alpaca, ExportFormat::Alpaca);
    assert_eq!(ExportFormat::Parquet, ExportFormat::Parquet);
    assert_ne!(ExportFormat::Jsonl, ExportFormat::Parquet);
    assert_ne!(ExportFormat::ChatML, ExportFormat::Alpaca);
}

#[test]
fn test_export_format_clone() {
    let fmt = ExportFormat::Parquet;
    let cloned = fmt;
    assert_eq!(cloned, ExportFormat::Parquet);
}

#[test]
fn test_export_format_copy() {
    let fmt = ExportFormat::ChatML;
    let copied = fmt;
    assert_eq!(fmt, copied);
}

// ============================================================================
// Stats: unverified examples
// ============================================================================

#[test]
fn test_stats_with_unverified_examples() {
    let mut exporter = DatasetExporter::empty();

    // Mix of verified and unverified
    exporter.add_example(TrainingExample {
        error_code: "E0308".to_string(),
        decision: "type_coercion".to_string(),
        fix_diff: "- old\n+ new".to_string(),
        description: "Verified pattern".to_string(),
        source: "bootstrap".to_string(),
        verified: true,
        success_count: 10,
        failure_count: 0,
    });

    exporter.add_example(TrainingExample {
        error_code: "E0308".to_string(),
        decision: "type_coercion".to_string(),
        fix_diff: "- old2\n+ new2".to_string(),
        description: "Unverified pattern".to_string(),
        source: "training".to_string(),
        verified: false,
        success_count: 1,
        failure_count: 5,
    });

    exporter.add_example(TrainingExample {
        error_code: "E0133".to_string(),
        decision: "unsafe_block".to_string(),
        fix_diff: "- *ptr\n+ unsafe { *ptr }".to_string(),
        description: "Another verified".to_string(),
        source: "imported".to_string(),
        verified: true,
        success_count: 3,
        failure_count: 1,
    });

    let stats = exporter.stats();
    assert_eq!(stats.total, 3);
    assert_eq!(stats.verified, 2);

    // error_code distribution
    assert_eq!(*stats.by_error_code.get("E0308").unwrap(), 2);
    assert_eq!(*stats.by_error_code.get("E0133").unwrap(), 1);

    // decision distribution
    assert_eq!(*stats.by_decision.get("type_coercion").unwrap(), 2);
    assert_eq!(*stats.by_decision.get("unsafe_block").unwrap(), 1);

    // source distribution
    assert_eq!(*stats.by_source.get("bootstrap").unwrap(), 1);
    assert_eq!(*stats.by_source.get("training").unwrap(), 1);
    assert_eq!(*stats.by_source.get("imported").unwrap(), 1);
}

// ============================================================================
// Export content verification: ChatML format details
// ============================================================================

#[test]
fn test_export_chatml_content_structure() {
    let mut exporter = DatasetExporter::empty();
    exporter.add_example(TrainingExample {
        error_code: "E0308".to_string(),
        decision: "type_coercion".to_string(),
        fix_diff: "- let x = val;\n+ let x = val as i32;".to_string(),
        description: "Add cast".to_string(),
        source: "test".to_string(),
        verified: true,
        success_count: 0,
        failure_count: 0,
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("chatml.jsonl");
    let count = exporter.export_chatml(&path).unwrap();
    assert_eq!(count, 1);

    let content = std::fs::read_to_string(&path).unwrap();
    let conv: ChatMLConversation = serde_json::from_str(content.trim()).unwrap();

    // User message contains error info
    assert!(conv.messages[0].content.contains("E0308"));
    assert!(conv.messages[0].content.contains("Add cast"));
    // User message contains lines starting with -
    assert!(conv.messages[0].content.contains("let x = val;"));

    // Assistant message contains fix
    assert!(conv.messages[1].content.contains("type_coercion"));
    assert!(conv.messages[1].content.contains("let x = val as i32;"));
}

// ============================================================================
// Export content verification: Alpaca format details
// ============================================================================

#[test]
fn test_export_alpaca_content_structure() {
    let mut exporter = DatasetExporter::empty();
    exporter.add_example(TrainingExample {
        error_code: "E0499".to_string(),
        decision: "remove_mut_alias".to_string(),
        fix_diff: "- let a = &mut x;\n- let b = &mut x;\n+ let a = &mut x;\n+ let b = &x;"
            .to_string(),
        description: "Remove double mutable borrow".to_string(),
        source: "test".to_string(),
        verified: true,
        success_count: 0,
        failure_count: 0,
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("alpaca.jsonl");
    let count = exporter.export_alpaca(&path).unwrap();
    assert_eq!(count, 1);

    let content = std::fs::read_to_string(&path).unwrap();
    let alpaca: AlpacaExample = serde_json::from_str(content.trim()).unwrap();

    // Instruction contains error code and description
    assert!(alpaca.instruction.contains("E0499"));
    assert!(alpaca.instruction.contains("Remove double mutable borrow"));

    // Input is the removed lines (-)
    assert!(alpaca.input.contains("let a = &mut x;"));
    assert!(alpaca.input.contains("let b = &mut x;"));

    // Output is the added lines (+)
    assert!(alpaca.output.contains("let a = &mut x;"));
    assert!(alpaca.output.contains("let b = &x;"));
}

// ============================================================================
// DatasetStats: clone trait
// ============================================================================

#[test]
fn test_dataset_stats_clone() {
    let mut by_error_code = HashMap::new();
    by_error_code.insert("E0308".to_string(), 5);

    let stats = DatasetStats {
        total: 5,
        verified: 3,
        by_error_code,
        by_decision: HashMap::new(),
        by_source: HashMap::new(),
    };

    let cloned = stats.clone();
    assert_eq!(cloned.total, 5);
    assert_eq!(cloned.verified, 3);
    assert_eq!(*cloned.by_error_code.get("E0308").unwrap(), 5);
}

#[test]
fn test_dataset_stats_debug() {
    let stats = DatasetStats {
        total: 10,
        verified: 7,
        by_error_code: HashMap::new(),
        by_decision: HashMap::new(),
        by_source: HashMap::new(),
    };
    let debug = format!("{:?}", stats);
    assert!(debug.contains("total: 10"));
    assert!(debug.contains("verified: 7"));
}

// ============================================================================
// Multiple examples with different sources for stats
// ============================================================================

#[test]
fn test_stats_all_unverified() {
    let mut exporter = DatasetExporter::empty();

    for i in 0..5 {
        exporter.add_example(TrainingExample {
            error_code: format!("E{:04}", 100 + i),
            decision: format!("decision_{}", i),
            fix_diff: format!("- old_{}\n+ new_{}", i, i),
            description: format!("Desc {}", i),
            source: "synthetic".to_string(),
            verified: false,
            success_count: 0,
            failure_count: 0,
        });
    }

    let stats = exporter.stats();
    assert_eq!(stats.total, 5);
    assert_eq!(stats.verified, 0);

    let md = stats.to_markdown();
    assert!(md.contains("0.0%")); // 0% verification rate
}

// ============================================================================
// JSONL format: verify each line is individually valid JSON
// ============================================================================

#[test]
fn test_export_jsonl_single_example() {
    let mut exporter = DatasetExporter::empty();
    exporter.add_example(TrainingExample {
        error_code: "E0515".to_string(),
        decision: "return_owned".to_string(),
        fix_diff: "- &local\n+ local.clone()".to_string(),
        description: "Return owned value instead of reference to local".to_string(),
        source: "bootstrap".to_string(),
        verified: true,
        success_count: 7,
        failure_count: 0,
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("single.jsonl");
    let count = exporter.export_jsonl(&path).unwrap();
    assert_eq!(count, 1);

    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: TrainingExample = serde_json::from_str(content.trim()).unwrap();
    assert_eq!(parsed.error_code, "E0515");
    assert_eq!(parsed.decision, "return_owned");
    assert_eq!(parsed.success_count, 7);
}

// ============================================================================
// Stats markdown: decision types sorted by count (descending)
// ============================================================================

#[test]
fn test_stats_markdown_decisions_sorted_by_count_desc() {
    let mut by_decision = HashMap::new();
    by_decision.insert("rare_decision".to_string(), 1);
    by_decision.insert("common_decision".to_string(), 100);
    by_decision.insert("medium_decision".to_string(), 50);

    let stats = DatasetStats {
        total: 151,
        verified: 151,
        by_error_code: HashMap::new(),
        by_decision,
        by_source: HashMap::new(),
    };

    let md = stats.to_markdown();
    // common should appear before medium, which should appear before rare
    let common_pos = md.find("common_decision").unwrap();
    let medium_pos = md.find("medium_decision").unwrap();
    let rare_pos = md.find("rare_decision").unwrap();
    assert!(common_pos < medium_pos);
    assert!(medium_pos < rare_pos);
}

// ============================================================================
// Stats markdown: sources sorted by count (descending)
// ============================================================================

#[test]
fn test_stats_markdown_sources_sorted_by_count_desc() {
    let mut by_source = HashMap::new();
    by_source.insert("bootstrap".to_string(), 50);
    by_source.insert("imported".to_string(), 5);
    by_source.insert("training".to_string(), 30);

    let stats = DatasetStats {
        total: 85,
        verified: 60,
        by_error_code: HashMap::new(),
        by_decision: HashMap::new(),
        by_source,
    };

    let md = stats.to_markdown();
    let bootstrap_pos = md.find("bootstrap").unwrap();
    let training_pos = md.find("training").unwrap();
    let imported_pos = md.find("imported").unwrap();
    assert!(bootstrap_pos < training_pos);
    assert!(training_pos < imported_pos);
}
