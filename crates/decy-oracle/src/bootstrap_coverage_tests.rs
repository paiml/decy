//! Coverage tests for get_bootstrap_patterns() in bootstrap.rs
//!
//! These tests ensure full line coverage of all 25 bootstrap patterns
//! by individually verifying each pattern's fields and exercising
//! all code paths through BootstrapStats methods.

use crate::bootstrap::*;
use std::collections::HashSet;

// ============================================================================
// CORE FUNCTION: Verify get_bootstrap_patterns returns correct count and shape
// ============================================================================

#[test]
fn bootstrap_patterns_returns_25_patterns() {
    let patterns = get_bootstrap_patterns();
    assert_eq!(patterns.len(), 25);
}

#[test]
fn bootstrap_patterns_all_fields_non_empty() {
    for p in get_bootstrap_patterns() {
        assert!(!p.error_code.is_empty(), "error_code empty for: {}", p.description);
        assert!(!p.fix_diff.is_empty(), "fix_diff empty for: {}", p.description);
        assert!(!p.decision.is_empty(), "decision empty for: {}", p.description);
        assert!(!p.description.is_empty(), "description empty");
    }
}

// ============================================================================
// PER-PATTERN VERIFICATION: Exercise every struct literal construction line
// ============================================================================

#[test]
fn pattern_idx_0_type_coercion_int_cast() {
    let p = &get_bootstrap_patterns()[0];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "type_coercion");
    assert!(p.fix_diff.contains("as i32"), "fix_diff: {}", p.fix_diff);
    assert!(p.description.contains("explicit type cast"), "desc: {}", p.description);
}

#[test]
fn pattern_idx_1_pointer_to_mut_reference() {
    let p = &get_bootstrap_patterns()[1];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "pointer_to_reference");
    assert!(p.fix_diff.contains("*mut i32"));
    assert!(p.fix_diff.contains("&mut i32"));
    assert!(p.description.contains("mutable reference"));
}

#[test]
fn pattern_idx_2_pointer_to_immut_reference() {
    let p = &get_bootstrap_patterns()[2];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "pointer_to_reference");
    assert!(p.fix_diff.contains("*const i32"));
    assert!(p.fix_diff.contains("&i32"));
    assert!(p.description.contains("immutable reference"));
}

#[test]
fn pattern_idx_3_mutable_reference_swap() {
    let p = &get_bootstrap_patterns()[3];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "mutable_reference");
    assert!(p.fix_diff.contains("&mut x"), "fix_diff: {}", p.fix_diff);
    assert!(p.fix_diff.contains("&mut y"), "fix_diff: {}", p.fix_diff);
}

#[test]
fn pattern_idx_4_exit_type_coercion() {
    let p = &get_bootstrap_patterns()[4];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "type_coercion");
    assert!(p.fix_diff.contains("std::process::exit"));
    assert!(p.fix_diff.contains("as i32"));
}

#[test]
fn pattern_idx_5_unsafe_deref_write() {
    let p = &get_bootstrap_patterns()[5];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_deref");
    assert!(p.fix_diff.contains("*ptr = value"));
    assert!(p.fix_diff.contains("unsafe"));
    assert!(p.description.contains("dereference"));
}

#[test]
fn pattern_idx_6_unsafe_deref_read() {
    let p = &get_bootstrap_patterns()[6];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_deref");
    assert!(p.fix_diff.contains("let x = *ptr"));
    assert!(p.fix_diff.contains("unsafe"));
    assert!(p.description.contains("pointer read"));
}

#[test]
fn pattern_idx_7_unsafe_extern_call() {
    let p = &get_bootstrap_patterns()[7];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_extern");
    assert!(p.fix_diff.contains("extern_fn()"));
    assert!(p.fix_diff.contains("unsafe"));
    assert!(p.description.contains("extern function"));
}

#[test]
fn pattern_idx_8_clone_before_move() {
    let p = &get_bootstrap_patterns()[8];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "clone_before_move");
    assert!(p.fix_diff.contains("value.clone()"));
    assert!(p.description.contains("Clone"));
}

#[test]
fn pattern_idx_9_borrow_instead_of_move() {
    let p = &get_bootstrap_patterns()[9];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "borrow_instead_of_move");
    assert!(p.fix_diff.contains("let y = &x"));
    assert!(p.description.contains("Borrow"));
}

#[test]
fn pattern_idx_10_borrow_parameter() {
    let p = &get_bootstrap_patterns()[10];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "borrow_parameter");
    assert!(p.fix_diff.contains("fn take(s: &String)"));
    assert!(p.description.contains("borrow"));
}

#[test]
fn pattern_idx_11_sequential_mutable_borrow() {
    let p = &get_bootstrap_patterns()[11];
    assert_eq!(p.error_code, "E0499");
    assert_eq!(p.decision, "sequential_mutable_borrow");
    assert!(p.fix_diff.contains("drop(a)"));
    assert!(p.description.contains("End first mutable borrow"));
}

#[test]
fn pattern_idx_12_use_stdlib_swap() {
    let p = &get_bootstrap_patterns()[12];
    assert_eq!(p.error_code, "E0499");
    assert_eq!(p.decision, "use_stdlib_method");
    assert!(p.fix_diff.contains("arr.swap(i, j)"));
    assert!(p.description.contains("stdlib"));
}

#[test]
fn pattern_idx_13_reorder_borrow() {
    let p = &get_bootstrap_patterns()[13];
    assert_eq!(p.error_code, "E0506");
    assert_eq!(p.decision, "reorder_borrow");
    assert!(p.fix_diff.contains("x = 5"));
    assert!(p.description.contains("Reorder"));
}

#[test]
fn pattern_idx_14_extend_lifetime() {
    let p = &get_bootstrap_patterns()[14];
    assert_eq!(p.error_code, "E0597");
    assert_eq!(p.decision, "extend_lifetime");
    assert!(p.fix_diff.contains("let x = 5"));
    assert!(p.description.contains("outer scope"));
}

#[test]
fn pattern_idx_15_return_owned_e0597() {
    let p = &get_bootstrap_patterns()[15];
    assert_eq!(p.error_code, "E0597");
    assert_eq!(p.decision, "return_owned");
    assert!(p.fix_diff.contains("-> i32"));
    assert!(p.description.contains("owned value"));
}

#[test]
fn pattern_idx_16_return_owned_e0515() {
    let p = &get_bootstrap_patterns()[16];
    assert_eq!(p.error_code, "E0515");
    assert_eq!(p.decision, "return_owned");
    assert!(p.fix_diff.contains("-> Vec<i32>"));
    assert!(p.description.contains("owned value"));
}

#[test]
fn pattern_idx_17_clone_return() {
    let p = &get_bootstrap_patterns()[17];
    assert_eq!(p.error_code, "E0515");
    assert_eq!(p.decision, "clone_return");
    assert!(p.fix_diff.contains("local.clone()"));
    assert!(p.description.contains("Clone"));
}

#[test]
fn pattern_idx_18_array_to_slice() {
    let p = &get_bootstrap_patterns()[18];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "array_to_slice");
    assert!(p.fix_diff.contains("&[i32]"));
    assert!(p.description.contains("slice"));
}

#[test]
fn pattern_idx_19_bounds_checked_access() {
    let p = &get_bootstrap_patterns()[19];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "bounds_checked_access");
    assert!(p.fix_diff.contains("arr.get(i)"));
    assert!(p.fix_diff.contains("unwrap_or(0)"));
    assert!(p.description.contains("bounds check"));
}

#[test]
fn pattern_idx_20_safe_pointer_arithmetic() {
    let p = &get_bootstrap_patterns()[20];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "safe_pointer_arithmetic");
    assert!(p.fix_diff.contains("wrapping_add(offset)"));
    assert!(p.description.contains("safe pointer arithmetic"));
}

#[test]
fn pattern_idx_21_malloc_to_box() {
    let p = &get_bootstrap_patterns()[21];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "malloc_to_box");
    assert!(p.fix_diff.contains("malloc"));
    assert!(p.fix_diff.contains("Box::new(value)"));
    assert!(p.description.contains("Box"));
}

#[test]
fn pattern_idx_22_malloc_array_to_vec() {
    let p = &get_bootstrap_patterns()[22];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "malloc_array_to_vec");
    assert!(p.fix_diff.contains("Vec::with_capacity(n)"));
    assert!(p.description.contains("Vec"));
}

#[test]
fn pattern_idx_23_arrow_to_dot() {
    let p = &get_bootstrap_patterns()[23];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "arrow_to_dot");
    assert!(p.fix_diff.contains("p->field"));
    assert!(p.fix_diff.contains("p.field"));
    assert!(p.description.contains("arrow"));
}

#[test]
fn pattern_idx_24_nullable_to_option() {
    let p = &get_bootstrap_patterns()[24];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "nullable_to_option");
    assert!(p.fix_diff.contains("Option<Box<Node>>"));
    assert!(p.description.contains("Option"));
}

// ============================================================================
// ERROR CODE CATEGORY VERIFICATION: Groups match expected counts
// ============================================================================

#[test]
fn error_codes_e0308_has_12_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0308")
        .count();
    assert_eq!(count, 12);
}

#[test]
fn error_codes_e0133_has_3_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0133")
        .count();
    assert_eq!(count, 3);
}

#[test]
fn error_codes_e0382_has_3_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0382")
        .count();
    assert_eq!(count, 3);
}

#[test]
fn error_codes_e0499_has_2_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0499")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn error_codes_e0506_has_1_pattern() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0506")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn error_codes_e0597_has_2_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0597")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn error_codes_e0515_has_2_patterns() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0515")
        .count();
    assert_eq!(count, 2);
}

// ============================================================================
// DECISION TYPES: Verify all 21 unique decisions present
// ============================================================================

#[test]
fn all_21_decision_types_present() {
    let patterns = get_bootstrap_patterns();
    let decisions: HashSet<&str> = patterns.iter().map(|p| p.decision).collect();
    assert_eq!(decisions.len(), 21);

    let expected = [
        "type_coercion",
        "pointer_to_reference",
        "mutable_reference",
        "unsafe_deref",
        "unsafe_extern",
        "clone_before_move",
        "borrow_instead_of_move",
        "borrow_parameter",
        "sequential_mutable_borrow",
        "use_stdlib_method",
        "reorder_borrow",
        "extend_lifetime",
        "return_owned",
        "clone_return",
        "array_to_slice",
        "bounds_checked_access",
        "safe_pointer_arithmetic",
        "malloc_to_box",
        "malloc_array_to_vec",
        "arrow_to_dot",
        "nullable_to_option",
    ];

    for d in &expected {
        assert!(decisions.contains(d), "Missing decision type: {}", d);
    }
}

// ============================================================================
// PATTERN FORMAT VALIDATION
// ============================================================================

#[test]
fn all_fix_diffs_contain_both_minus_and_plus() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.fix_diff.contains('-') && p.fix_diff.contains('+'),
            "Pattern '{}' fix_diff missing - or +: {}",
            p.decision,
            p.fix_diff
        );
    }
}

#[test]
fn all_error_codes_are_exxxx_format() {
    for p in get_bootstrap_patterns() {
        assert!(p.error_code.starts_with('E'), "error_code should start with E: {}", p.error_code);
        assert_eq!(p.error_code.len(), 5, "error_code should be 5 chars: {}", p.error_code);
        assert!(
            p.error_code[1..].chars().all(|c| c.is_ascii_digit()),
            "error_code digits: {}",
            p.error_code
        );
    }
}

#[test]
fn all_decisions_are_snake_case() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.decision.chars().all(|c| c.is_lowercase() || c == '_'),
            "decision not snake_case: {}",
            p.decision
        );
    }
}

#[test]
fn all_descriptions_start_with_uppercase() {
    for p in get_bootstrap_patterns() {
        let first = p.description.chars().next().unwrap();
        assert!(first.is_uppercase(), "description should start uppercase: {}", p.description);
    }
}

// ============================================================================
// BOOTSTRAP PATTERN: Clone and Debug traits
// ============================================================================

#[test]
fn bootstrap_pattern_clone_preserves_fields() {
    let patterns = get_bootstrap_patterns();
    for p in &patterns {
        let cloned = p.clone();
        assert_eq!(cloned.error_code, p.error_code);
        assert_eq!(cloned.fix_diff, p.fix_diff);
        assert_eq!(cloned.decision, p.decision);
        assert_eq!(cloned.description, p.description);
    }
}

#[test]
fn bootstrap_pattern_debug_contains_struct_name() {
    let patterns = get_bootstrap_patterns();
    let debug = format!("{:?}", patterns[0]);
    assert!(debug.contains("BootstrapPattern"));
}

// ============================================================================
// BOOTSTRAP STATS: Comprehensive verification
// ============================================================================

#[test]
fn stats_from_patterns_total() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.total_patterns, 25);
}

#[test]
fn stats_from_patterns_error_code_map() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.by_error_code.len(), 7);
    assert_eq!(*stats.by_error_code.get("E0308").unwrap(), 12);
    assert_eq!(*stats.by_error_code.get("E0133").unwrap(), 3);
    assert_eq!(*stats.by_error_code.get("E0382").unwrap(), 3);
    assert_eq!(*stats.by_error_code.get("E0499").unwrap(), 2);
    assert_eq!(*stats.by_error_code.get("E0506").unwrap(), 1);
    assert_eq!(*stats.by_error_code.get("E0597").unwrap(), 2);
    assert_eq!(*stats.by_error_code.get("E0515").unwrap(), 2);
}

#[test]
fn stats_from_patterns_decision_map() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.by_decision.len(), 21);
    // Decisions with count > 1
    assert_eq!(*stats.by_decision.get("type_coercion").unwrap(), 2);
    assert_eq!(*stats.by_decision.get("pointer_to_reference").unwrap(), 2);
    assert_eq!(*stats.by_decision.get("unsafe_deref").unwrap(), 2);
    assert_eq!(*stats.by_decision.get("return_owned").unwrap(), 2);
    // Decisions with count = 1
    assert_eq!(*stats.by_decision.get("mutable_reference").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("unsafe_extern").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("clone_before_move").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("borrow_instead_of_move").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("borrow_parameter").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("sequential_mutable_borrow").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("use_stdlib_method").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("reorder_borrow").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("extend_lifetime").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("clone_return").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("array_to_slice").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("bounds_checked_access").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("safe_pointer_arithmetic").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("malloc_to_box").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("malloc_array_to_vec").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("arrow_to_dot").unwrap(), 1);
    assert_eq!(*stats.by_decision.get("nullable_to_option").unwrap(), 1);
}

#[test]
fn stats_default_is_empty() {
    let stats = BootstrapStats::default();
    assert_eq!(stats.total_patterns, 0);
    assert!(stats.by_error_code.is_empty());
    assert!(stats.by_decision.is_empty());
}

#[test]
fn stats_debug_format() {
    let stats = BootstrapStats::from_patterns();
    let debug = format!("{:?}", stats);
    assert!(debug.contains("BootstrapStats"));
    assert!(debug.contains("total_patterns"));
    assert!(debug.contains("by_error_code"));
    assert!(debug.contains("by_decision"));
}

// ============================================================================
// PRETTY FORMAT: Content and ordering
// ============================================================================

#[test]
fn stats_pretty_contains_header() {
    let stats = BootstrapStats::from_patterns();
    let pretty = stats.to_string_pretty();
    assert!(pretty.contains("Bootstrap Patterns: 25"));
    assert!(pretty.contains("By Error Code:"));
    assert!(pretty.contains("By Decision Type:"));
}

#[test]
fn stats_pretty_contains_all_error_codes_with_counts() {
    let stats = BootstrapStats::from_patterns();
    let pretty = stats.to_string_pretty();
    assert!(pretty.contains("E0308: 12"));
    assert!(pretty.contains("E0133: 3"));
    assert!(pretty.contains("E0382: 3"));
    assert!(pretty.contains("E0499: 2"));
    assert!(pretty.contains("E0506: 1"));
    assert!(pretty.contains("E0597: 2"));
    assert!(pretty.contains("E0515: 2"));
}

#[test]
fn stats_pretty_error_codes_sorted_alphabetically() {
    let stats = BootstrapStats::from_patterns();
    let pretty = stats.to_string_pretty();
    let pos_e0133 = pretty.find("E0133").unwrap();
    let pos_e0308 = pretty.find("E0308").unwrap();
    let pos_e0382 = pretty.find("E0382").unwrap();
    let pos_e0499 = pretty.find("E0499").unwrap();
    let pos_e0506 = pretty.find("E0506").unwrap();
    let pos_e0515 = pretty.find("E0515").unwrap();
    let pos_e0597 = pretty.find("E0597").unwrap();
    assert!(pos_e0133 < pos_e0308);
    assert!(pos_e0308 < pos_e0382);
    assert!(pos_e0382 < pos_e0499);
    assert!(pos_e0499 < pos_e0506);
    assert!(pos_e0506 < pos_e0515);
    assert!(pos_e0515 < pos_e0597);
}

#[test]
fn stats_pretty_decisions_sorted_by_count_desc() {
    let stats = BootstrapStats::from_patterns();
    let pretty = stats.to_string_pretty();
    let decision_section = pretty.find("By Decision Type:").unwrap();
    let tail = &pretty[decision_section..];
    // Decisions with count 2 should appear before decisions with count 1
    let pos_type_coercion = tail.find("type_coercion").unwrap();
    let pos_nullable = tail.find("nullable_to_option").unwrap();
    assert!(
        pos_type_coercion < pos_nullable,
        "type_coercion (2) should appear before nullable_to_option (1)"
    );
}

// ============================================================================
// CATEGORY-SPECIFIC PATTERN CONTENT VALIDATION
// ============================================================================

#[test]
fn c_specific_array_pointer_patterns_have_correct_content() {
    let patterns = get_bootstrap_patterns();
    // Patterns 18-20 are C-specific array/pointer patterns
    let array_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| {
            p.decision == "array_to_slice"
                || p.decision == "bounds_checked_access"
                || p.decision == "safe_pointer_arithmetic"
        })
        .collect();
    assert_eq!(array_patterns.len(), 3);
    assert!(array_patterns.iter().all(|p| p.error_code == "E0308"));
}

#[test]
fn c_specific_malloc_free_patterns_have_correct_content() {
    let patterns = get_bootstrap_patterns();
    let malloc_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| p.decision == "malloc_to_box" || p.decision == "malloc_array_to_vec")
        .collect();
    assert_eq!(malloc_patterns.len(), 2);
    assert!(malloc_patterns.iter().all(|p| p.error_code == "E0308"));
    assert!(malloc_patterns.iter().any(|p| p.fix_diff.contains("Box::new")));
    assert!(malloc_patterns.iter().any(|p| p.fix_diff.contains("Vec::with_capacity")));
}

#[test]
fn c_specific_struct_patterns_have_correct_content() {
    let patterns = get_bootstrap_patterns();
    let struct_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| p.decision == "arrow_to_dot" || p.decision == "nullable_to_option")
        .collect();
    assert_eq!(struct_patterns.len(), 2);
    assert!(struct_patterns.iter().all(|p| p.error_code == "E0308"));
}

#[test]
fn e0133_unsafe_patterns_all_mention_unsafe_in_diff() {
    let patterns = get_bootstrap_patterns();
    let unsafe_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| p.error_code == "E0133")
        .collect();
    for p in &unsafe_patterns {
        assert!(
            p.fix_diff.contains("unsafe"),
            "E0133 pattern should mention 'unsafe': {}",
            p.description
        );
    }
}

#[test]
fn e0382_ownership_patterns_cover_all_strategies() {
    let patterns = get_bootstrap_patterns();
    let e0382: Vec<_> = patterns
        .iter()
        .filter(|p| p.error_code == "E0382")
        .collect();
    let decisions: HashSet<&str> = e0382.iter().map(|p| p.decision).collect();
    assert!(decisions.contains("clone_before_move"));
    assert!(decisions.contains("borrow_instead_of_move"));
    assert!(decisions.contains("borrow_parameter"));
}

#[test]
fn e0499_borrow_patterns_cover_both_strategies() {
    let patterns = get_bootstrap_patterns();
    let e0499: Vec<_> = patterns
        .iter()
        .filter(|p| p.error_code == "E0499")
        .collect();
    let decisions: HashSet<&str> = e0499.iter().map(|p| p.decision).collect();
    assert!(decisions.contains("sequential_mutable_borrow"));
    assert!(decisions.contains("use_stdlib_method"));
}

// ============================================================================
// COMBINED ITERATION: Force full vec construction
// ============================================================================

#[test]
fn iterate_all_patterns_collect_all_fields() {
    let patterns = get_bootstrap_patterns();
    let mut error_codes = Vec::with_capacity(25);
    let mut fix_diffs = Vec::with_capacity(25);
    let mut decisions = Vec::with_capacity(25);
    let mut descriptions = Vec::with_capacity(25);

    for p in &patterns {
        error_codes.push(p.error_code);
        fix_diffs.push(p.fix_diff);
        decisions.push(p.decision);
        descriptions.push(p.description);
    }

    assert_eq!(error_codes.len(), 25);
    assert_eq!(fix_diffs.len(), 25);
    assert_eq!(decisions.len(), 25);
    assert_eq!(descriptions.len(), 25);

    // Force all collected values to be used (prevents dead code elimination)
    let total_chars: usize = error_codes.iter().map(|s| s.len()).sum::<usize>()
        + fix_diffs.iter().map(|s| s.len()).sum::<usize>()
        + decisions.iter().map(|s| s.len()).sum::<usize>()
        + descriptions.iter().map(|s| s.len()).sum::<usize>();
    assert!(total_chars > 500, "Should have substantial combined content");
}

// ============================================================================
// UNIQUENESS: No duplicate patterns
// ============================================================================

#[test]
fn no_duplicate_decision_description_pairs() {
    let patterns = get_bootstrap_patterns();
    let pairs: HashSet<_> = patterns
        .iter()
        .map(|p| (p.decision, p.description))
        .collect();
    assert_eq!(
        pairs.len(),
        patterns.len(),
        "Each pattern should have unique (decision, description) pair"
    );
}

#[test]
fn no_duplicate_fix_diffs() {
    let patterns = get_bootstrap_patterns();
    let diffs: HashSet<_> = patterns.iter().map(|p| p.fix_diff).collect();
    assert_eq!(
        diffs.len(),
        patterns.len(),
        "Each pattern should have a unique fix_diff"
    );
}
