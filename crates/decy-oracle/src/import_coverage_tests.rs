//! Coverage tests for import.rs smart_import_filter function.
//!
//! Targets uncovered branches in:
//! - smart_import_filter() (line 210+): every match arm and conditional
//! - analyze_fix_strategy(): all pattern branches
//! - ImportStats: record/acceptance_rate/overall_acceptance_rate
//! - ImportDecision: allows_import for all variants

use crate::import::{
    analyze_fix_strategy, smart_import_filter, FixStrategy, ImportDecision, ImportStats,
    SmartImportConfig, SourceLanguage,
};
use std::collections::HashMap;

// ============================================================================
// analyze_fix_strategy() branch coverage
// ============================================================================

#[test]
fn strategy_clone_via_dot_clone() {
    assert_eq!(
        analyze_fix_strategy("let x = value.clone();"),
        FixStrategy::AddClone
    );
}

#[test]
fn strategy_clone_via_to_owned() {
    assert_eq!(
        analyze_fix_strategy("let s = slice.to_owned();"),
        FixStrategy::AddClone
    );
}

#[test]
fn strategy_lifetime_via_angle_bracket_a() {
    assert_eq!(
        analyze_fix_strategy("fn foo<'a>(x: &'a str) -> &'a str"),
        FixStrategy::AddLifetime
    );
}

#[test]
fn strategy_lifetime_via_static() {
    assert_eq!(
        analyze_fix_strategy("let s: &'static str = \"hello\";"),
        FixStrategy::AddLifetime
    );
}

#[test]
fn strategy_lifetime_via_underscore() {
    assert_eq!(
        analyze_fix_strategy("fn bar(x: &'_ str)"),
        FixStrategy::AddLifetime
    );
}

#[test]
fn strategy_lifetime_via_tick_a_in_fn() {
    assert_eq!(
        analyze_fix_strategy("fn process<'a>(data: &'a [u8])"),
        FixStrategy::AddLifetime
    );
}

#[test]
fn strategy_borrow_via_ref_type() {
    assert_eq!(
        analyze_fix_strategy("fn foo(x: &String)"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_mut_ref() {
    assert_eq!(
        analyze_fix_strategy("fn foo(x: &mut Vec<i32>)"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_self_ref() {
    assert_eq!(
        analyze_fix_strategy("fn method(&self) -> i32"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_mut_self() {
    assert_eq!(
        analyze_fix_strategy("fn method(&mut self) { }"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_param_x_ref() {
    assert_eq!(
        analyze_fix_strategy("fn foo(x: &i32)"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_param_y_ref() {
    assert_eq!(
        analyze_fix_strategy("fn bar(y: &str)"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_param_z_ref() {
    assert_eq!(
        analyze_fix_strategy("fn baz(z: &[u8])"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_borrow_via_ampersand_plus_fn() {
    assert_eq!(
        analyze_fix_strategy("+ fn handle(data: &[u8]) {}\n& used"),
        FixStrategy::AddBorrow
    );
}

#[test]
fn strategy_option_via_option_type() {
    assert_eq!(
        analyze_fix_strategy("let x: Option<i32> = None;"),
        FixStrategy::WrapInOption
    );
}

#[test]
fn strategy_option_via_some() {
    assert_eq!(
        analyze_fix_strategy("let x = Some(42);"),
        FixStrategy::WrapInOption
    );
}

#[test]
fn strategy_option_via_unwrap() {
    assert_eq!(
        analyze_fix_strategy("let v = opt.unwrap();"),
        FixStrategy::WrapInOption
    );
}

#[test]
fn strategy_option_via_is_none() {
    assert_eq!(
        analyze_fix_strategy("if x.is_none() { return; }"),
        FixStrategy::WrapInOption
    );
}

#[test]
fn strategy_option_via_is_some() {
    assert_eq!(
        analyze_fix_strategy("if x.is_some() { process(x); }"),
        FixStrategy::WrapInOption
    );
}

#[test]
fn strategy_result_via_result_type() {
    assert_eq!(
        analyze_fix_strategy("fn foo() -> Result<i32, Error>"),
        FixStrategy::WrapInResult
    );
}

#[test]
fn strategy_result_via_ok() {
    assert_eq!(
        analyze_fix_strategy("return Ok(42);"),
        FixStrategy::WrapInResult
    );
}

#[test]
fn strategy_result_via_err() {
    assert_eq!(
        analyze_fix_strategy("return Err(\"failed\");"),
        FixStrategy::WrapInResult
    );
}

#[test]
fn strategy_type_annotation_via_i32() {
    assert_eq!(
        analyze_fix_strategy("let x: i32 = 5;"),
        FixStrategy::AddTypeAnnotation
    );
}

#[test]
fn strategy_type_annotation_via_string() {
    assert_eq!(
        analyze_fix_strategy("let s: String = String::new();"),
        FixStrategy::AddTypeAnnotation
    );
}

#[test]
fn strategy_type_annotation_generic_colon_space() {
    // ": " without "&" matches AddTypeAnnotation
    assert_eq!(
        analyze_fix_strategy("let v: Vec<u8> = vec![];"),
        FixStrategy::AddTypeAnnotation
    );
}

#[test]
fn strategy_unknown_for_gibberish() {
    assert_eq!(
        analyze_fix_strategy("random text without patterns"),
        FixStrategy::Unknown
    );
}

#[test]
fn strategy_unknown_empty_string() {
    assert_eq!(analyze_fix_strategy(""), FixStrategy::Unknown);
}

// ============================================================================
// smart_import_filter() branch coverage
// ============================================================================

fn python_config() -> SmartImportConfig {
    SmartImportConfig {
        source_language: SourceLanguage::Python,
        min_confidence: 0.5,
        allow_warnings: true,
    }
}

fn c_config() -> SmartImportConfig {
    SmartImportConfig {
        source_language: SourceLanguage::C,
        min_confidence: 0.5,
        allow_warnings: true,
    }
}

fn cpp_config() -> SmartImportConfig {
    SmartImportConfig {
        source_language: SourceLanguage::Cpp,
        min_confidence: 0.5,
        allow_warnings: true,
    }
}

fn other_config() -> SmartImportConfig {
    SmartImportConfig {
        source_language: SourceLanguage::Other,
        min_confidence: 0.5,
        allow_warnings: true,
    }
}

// --- AddClone branch ---

#[test]
fn filter_clone_python_with_list_construct_rejected() {
    let diff = "let x = lst.clone();";
    let mut meta = HashMap::new();
    meta.insert("source_construct".to_string(), "list_copy".to_string());
    let decision = smart_import_filter(diff, &meta, &python_config());
    assert!(matches!(decision, ImportDecision::Reject(_)));
    assert!(!decision.allows_import());
}

#[test]
fn filter_clone_python_with_dict_construct_rejected() {
    let diff = "let x = d.clone();";
    let mut meta = HashMap::new();
    meta.insert("source_construct".to_string(), "dict_merge".to_string());
    let decision = smart_import_filter(diff, &meta, &python_config());
    assert!(matches!(decision, ImportDecision::Reject(_)));
}

#[test]
fn filter_clone_python_no_construct_accepted() {
    let diff = "let x = val.clone();";
    let meta = HashMap::new();
    let decision = smart_import_filter(diff, &meta, &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_clone_python_non_collection_construct_accepted() {
    let diff = "let x = val.clone();";
    let mut meta = HashMap::new();
    meta.insert("source_construct".to_string(), "integer_copy".to_string());
    let decision = smart_import_filter(diff, &meta, &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_clone_c_source_always_accepted() {
    let diff = "let x = val.clone();";
    let mut meta = HashMap::new();
    meta.insert("source_construct".to_string(), "list_copy".to_string());
    let decision = smart_import_filter(diff, &meta, &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- AddBorrow branch ---

#[test]
fn filter_borrow_python_accepted() {
    let diff = "fn foo(x: &String)";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_borrow_c_accepted() {
    let diff = "fn foo(x: &mut Vec<i32>)";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_borrow_cpp_accepted() {
    let diff = "fn foo(x: &i32)";
    let decision = smart_import_filter(diff, &HashMap::new(), &cpp_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- AddLifetime branch ---

#[test]
fn filter_lifetime_python_accepted() {
    let diff = "fn foo<'a>(x: &'a str)";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_lifetime_c_accepted() {
    let diff = "fn foo<'a>(x: &'a str)";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_lifetime_other_accepted() {
    let diff = "fn foo<'a>(x: &'a str)";
    let decision = smart_import_filter(diff, &HashMap::new(), &other_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- WrapInOption branch ---

#[test]
fn filter_option_python_without_null_warns() {
    let diff = "let x = Some(value);";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    match decision {
        ImportDecision::AcceptWithWarning(msg) => {
            assert!(msg.contains("NULL"));
        }
        _ => panic!("Expected AcceptWithWarning, got {:?}", decision),
    }
}

#[test]
fn filter_option_python_with_null_accepts() {
    let diff = "if ptr == NULL { None } else { Some(v) }";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_python_with_nullptr_accepts() {
    // Must contain an Option pattern for strategy detection AND nullptr for null handling
    let diff = "let x: Option<i32> = if ptr == nullptr { None } else { Some(v) };";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_python_with_lowercase_null_accepts() {
    let diff = "let x = if p == null { None } else { Some(p) };";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_python_with_is_none_accepts() {
    let diff = "if x.is_none() { handle_null(); }";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_python_with_is_some_accepts() {
    let diff = "if x.is_some() { process(); }";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_python_with_unwrap_or_accepts() {
    // Must contain an Option pattern for strategy detection AND unwrap_or for null handling
    let diff = "let v: Option<i32> = x; let r = v.unwrap_or(0);";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_c_source_always_accepted() {
    let diff = "let x = Some(value);";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_option_cpp_source_always_accepted() {
    let diff = "let x = Some(value);";
    let decision = smart_import_filter(diff, &HashMap::new(), &cpp_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- WrapInResult branch ---

#[test]
fn filter_result_python_accepted() {
    let diff = "fn process() -> Result<i32, Error>";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_result_c_accepted() {
    let diff = "return Ok(value);";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_result_err_variant() {
    let diff = "return Err(e);";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- AddTypeAnnotation branch ---

#[test]
fn filter_type_annotation_python_warns() {
    let diff = "let x: i32 = 5;";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    match decision {
        ImportDecision::AcceptWithWarning(msg) => {
            assert!(msg.contains("type mapping"));
        }
        _ => panic!(
            "Expected AcceptWithWarning for Python type annotation, got {:?}",
            decision
        ),
    }
}

#[test]
fn filter_type_annotation_c_accepted() {
    let diff = "let x: i32 = 5;";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_type_annotation_cpp_accepted() {
    let diff = "let x: i32 = 5;";
    let decision = smart_import_filter(diff, &HashMap::new(), &cpp_config());
    assert_eq!(decision, ImportDecision::Accept);
}

#[test]
fn filter_type_annotation_other_accepted() {
    let diff = "let x: i32 = 5;";
    let decision = smart_import_filter(diff, &HashMap::new(), &other_config());
    assert_eq!(decision, ImportDecision::Accept);
}

// --- Unknown strategy branch ---

#[test]
fn filter_unknown_rejected() {
    let diff = "random junk no patterns";
    let decision = smart_import_filter(diff, &HashMap::new(), &python_config());
    match decision {
        ImportDecision::Reject(reason) => {
            assert!(reason.contains("Unknown"));
        }
        _ => panic!("Expected Reject for unknown strategy"),
    }
}

#[test]
fn filter_unknown_rejected_c() {
    let diff = "no fix patterns here";
    let decision = smart_import_filter(diff, &HashMap::new(), &c_config());
    assert!(matches!(decision, ImportDecision::Reject(_)));
}

// ============================================================================
// ImportDecision::allows_import() coverage
// ============================================================================

#[test]
fn decision_accept_allows_import() {
    assert!(ImportDecision::Accept.allows_import());
}

#[test]
fn decision_accept_with_warning_allows_import() {
    assert!(ImportDecision::AcceptWithWarning("warn".to_string()).allows_import());
}

#[test]
fn decision_reject_does_not_allow_import() {
    assert!(!ImportDecision::Reject("reason".to_string()).allows_import());
}

// ============================================================================
// ImportStats coverage
// ============================================================================

#[test]
fn stats_new_is_empty() {
    let stats = ImportStats::new();
    assert_eq!(stats.total_evaluated, 0);
    assert_eq!(stats.warnings, 0);
    assert!(stats.accepted_by_strategy.is_empty());
    assert!(stats.rejected_by_strategy.is_empty());
}

#[test]
fn stats_record_accept_increments() {
    let mut stats = ImportStats::new();
    stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);

    assert_eq!(stats.total_evaluated, 1);
    assert_eq!(stats.warnings, 0);
    assert_eq!(
        stats.accepted_by_strategy.get(&FixStrategy::AddBorrow),
        Some(&1)
    );
}

#[test]
fn stats_record_warning_increments_both() {
    let mut stats = ImportStats::new();
    stats.record(
        FixStrategy::WrapInOption,
        &ImportDecision::AcceptWithWarning("check NULL".to_string()),
    );

    assert_eq!(stats.total_evaluated, 1);
    assert_eq!(stats.warnings, 1);
    assert_eq!(
        stats.accepted_by_strategy.get(&FixStrategy::WrapInOption),
        Some(&1)
    );
}

#[test]
fn stats_record_reject_increments() {
    let mut stats = ImportStats::new();
    stats.record(
        FixStrategy::AddClone,
        &ImportDecision::Reject("python collection".to_string()),
    );

    assert_eq!(stats.total_evaluated, 1);
    assert_eq!(stats.warnings, 0);
    assert_eq!(
        stats.rejected_by_strategy.get(&FixStrategy::AddClone),
        Some(&1)
    );
}

#[test]
fn stats_acceptance_rate_for_strategy() {
    let mut stats = ImportStats::new();
    stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
    stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
    stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
    stats.record(
        FixStrategy::AddBorrow,
        &ImportDecision::Reject("r".to_string()),
    );

    let rate = stats.acceptance_rate(FixStrategy::AddBorrow);
    assert!((rate - 0.75).abs() < 0.01);
}

#[test]
fn stats_acceptance_rate_zero_for_unseen_strategy() {
    let stats = ImportStats::new();
    assert_eq!(stats.acceptance_rate(FixStrategy::AddLifetime), 0.0);
}

#[test]
fn stats_overall_acceptance_rate_mixed() {
    let mut stats = ImportStats::new();
    stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
    stats.record(FixStrategy::AddClone, &ImportDecision::Accept);
    stats.record(
        FixStrategy::Unknown,
        &ImportDecision::Reject("unknown".to_string()),
    );
    stats.record(
        FixStrategy::WrapInOption,
        &ImportDecision::AcceptWithWarning("w".to_string()),
    );

    // 3 accepted (AddBorrow + AddClone + WrapInOption with warning) / 4 total
    let rate = stats.overall_acceptance_rate();
    assert!((rate - 0.75).abs() < 0.01);
}

#[test]
fn stats_overall_acceptance_rate_empty() {
    let stats = ImportStats::new();
    assert_eq!(stats.overall_acceptance_rate(), 0.0);
}

#[test]
fn stats_overall_acceptance_rate_all_accepted() {
    let mut stats = ImportStats::new();
    for _ in 0..10 {
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
    }
    assert!((stats.overall_acceptance_rate() - 1.0).abs() < 0.001);
}

#[test]
fn stats_overall_acceptance_rate_all_rejected() {
    let mut stats = ImportStats::new();
    for _ in 0..5 {
        stats.record(
            FixStrategy::Unknown,
            &ImportDecision::Reject("bad".to_string()),
        );
    }
    assert_eq!(stats.overall_acceptance_rate(), 0.0);
}

// ============================================================================
// SmartImportConfig coverage
// ============================================================================

#[test]
fn config_default_values() {
    let config = SmartImportConfig::default();
    assert_eq!(config.source_language, SourceLanguage::Python);
    assert!((config.min_confidence - 0.5).abs() < 0.01);
    assert!(config.allow_warnings);
}

#[test]
fn config_custom_values() {
    let config = SmartImportConfig {
        source_language: SourceLanguage::C,
        min_confidence: 0.8,
        allow_warnings: false,
    };
    assert_eq!(config.source_language, SourceLanguage::C);
    assert!((config.min_confidence - 0.8).abs() < 0.01);
    assert!(!config.allow_warnings);
}

// ============================================================================
// SourceLanguage coverage
// ============================================================================

#[test]
fn source_language_equality() {
    assert_eq!(SourceLanguage::Python, SourceLanguage::Python);
    assert_eq!(SourceLanguage::C, SourceLanguage::C);
    assert_eq!(SourceLanguage::Cpp, SourceLanguage::Cpp);
    assert_eq!(SourceLanguage::Other, SourceLanguage::Other);
    assert_ne!(SourceLanguage::Python, SourceLanguage::C);
}

// ============================================================================
// FixStrategy coverage
// ============================================================================

#[test]
fn fix_strategy_hash_and_eq() {
    let mut map: HashMap<FixStrategy, i32> = HashMap::new();
    map.insert(FixStrategy::AddClone, 1);
    map.insert(FixStrategy::AddBorrow, 2);
    map.insert(FixStrategy::AddLifetime, 3);
    map.insert(FixStrategy::WrapInOption, 4);
    map.insert(FixStrategy::WrapInResult, 5);
    map.insert(FixStrategy::AddTypeAnnotation, 6);
    map.insert(FixStrategy::Unknown, 7);

    assert_eq!(map.len(), 7);
    assert_eq!(map[&FixStrategy::AddClone], 1);
    assert_eq!(map[&FixStrategy::Unknown], 7);
}

#[test]
fn fix_strategy_debug() {
    let strategies = [
        FixStrategy::AddClone,
        FixStrategy::AddBorrow,
        FixStrategy::AddLifetime,
        FixStrategy::WrapInOption,
        FixStrategy::WrapInResult,
        FixStrategy::AddTypeAnnotation,
        FixStrategy::Unknown,
    ];
    for s in &strategies {
        let debug = format!("{:?}", s);
        assert!(!debug.is_empty());
    }
}

#[test]
fn fix_strategy_clone() {
    let original = FixStrategy::AddBorrow;
    let cloned = original;
    assert_eq!(original, cloned);
}

// ============================================================================
// ImportDecision edge cases
// ============================================================================

#[test]
fn import_decision_debug() {
    let decisions = [
        ImportDecision::Accept,
        ImportDecision::AcceptWithWarning("test warning".to_string()),
        ImportDecision::Reject("test reason".to_string()),
    ];
    for d in &decisions {
        let debug = format!("{:?}", d);
        assert!(!debug.is_empty());
    }
}

#[test]
fn import_decision_eq() {
    assert_eq!(ImportDecision::Accept, ImportDecision::Accept);
    assert_eq!(
        ImportDecision::AcceptWithWarning("a".to_string()),
        ImportDecision::AcceptWithWarning("a".to_string())
    );
    assert_ne!(
        ImportDecision::AcceptWithWarning("a".to_string()),
        ImportDecision::AcceptWithWarning("b".to_string())
    );
    assert_eq!(
        ImportDecision::Reject("x".to_string()),
        ImportDecision::Reject("x".to_string())
    );
    assert_ne!(ImportDecision::Accept, ImportDecision::Reject("r".to_string()));
}
