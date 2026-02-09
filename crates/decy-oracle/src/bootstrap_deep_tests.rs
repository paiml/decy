//! Deep coverage tests for bootstrap module.
//!
//! Exercises every code path in `get_bootstrap_patterns()` and related
//! `BootstrapStats` methods to drive line coverage from 0% toward 100%.

use super::bootstrap::{get_bootstrap_patterns, BootstrapStats};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Section 1: get_bootstrap_patterns() -- basic structure tests
// ============================================================================

#[test]
fn deep_bootstrap_returns_vec_not_empty() {
    let patterns = get_bootstrap_patterns();
    assert!(!patterns.is_empty());
}

#[test]
fn deep_bootstrap_exact_count_25() {
    let patterns = get_bootstrap_patterns();
    assert_eq!(patterns.len(), 25);
}

#[test]
fn deep_bootstrap_all_error_codes_start_with_e() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.error_code.starts_with('E'),
            "Bad error_code: {}",
            p.error_code
        );
    }
}

#[test]
fn deep_bootstrap_all_error_codes_are_5_chars() {
    for p in get_bootstrap_patterns() {
        assert_eq!(p.error_code.len(), 5, "Bad length: {}", p.error_code);
    }
}

#[test]
fn deep_bootstrap_all_error_codes_numeric_suffix() {
    for p in get_bootstrap_patterns() {
        let suffix = &p.error_code[1..];
        assert!(
            suffix.chars().all(|c| c.is_ascii_digit()),
            "Non-numeric suffix in: {}",
            p.error_code
        );
    }
}

// ============================================================================
// Section 2: error code coverage -- every known error code
// ============================================================================

#[test]
fn deep_e0308_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0308")
        .count();
    assert_eq!(count, 12);
}

#[test]
fn deep_e0133_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0133")
        .count();
    assert_eq!(count, 3);
}

#[test]
fn deep_e0382_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0382")
        .count();
    assert_eq!(count, 3);
}

#[test]
fn deep_e0499_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0499")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn deep_e0506_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0506")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn deep_e0597_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0597")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn deep_e0515_count() {
    let count = get_bootstrap_patterns()
        .iter()
        .filter(|p| p.error_code == "E0515")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn deep_exactly_7_distinct_error_codes() {
    let codes: HashSet<&str> = get_bootstrap_patterns()
        .iter()
        .map(|p| p.error_code)
        .collect();
    assert_eq!(codes.len(), 7);
}

#[test]
fn deep_error_code_sum_equals_total() {
    let patterns = get_bootstrap_patterns();
    let mut by_code: HashMap<&str, usize> = HashMap::new();
    for p in &patterns {
        *by_code.entry(p.error_code).or_default() += 1;
    }
    let sum: usize = by_code.values().sum();
    assert_eq!(sum, patterns.len());
}

// ============================================================================
// Section 3: decision category coverage
// ============================================================================

#[test]
fn deep_exactly_21_distinct_decisions() {
    let decisions: HashSet<&str> = get_bootstrap_patterns()
        .iter()
        .map(|p| p.decision)
        .collect();
    assert_eq!(decisions.len(), 21);
}

#[test]
fn deep_decision_type_coercion() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "type_coercion")
        .collect();
    assert_eq!(ps.len(), 2);
    assert!(ps.iter().all(|p| p.error_code == "E0308"));
}

#[test]
fn deep_decision_pointer_to_reference() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "pointer_to_reference")
        .collect();
    assert_eq!(ps.len(), 2);
    assert!(ps.iter().any(|p| p.fix_diff.contains("*mut")));
    assert!(ps.iter().any(|p| p.fix_diff.contains("*const")));
}

#[test]
fn deep_decision_mutable_reference() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "mutable_reference")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("&mut"));
}

#[test]
fn deep_decision_unsafe_deref() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "unsafe_deref")
        .collect();
    assert_eq!(ps.len(), 2);
    for p in &ps {
        assert_eq!(p.error_code, "E0133");
        assert!(p.fix_diff.contains("unsafe"));
    }
}

#[test]
fn deep_decision_unsafe_extern() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "unsafe_extern")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("extern_fn"));
}

#[test]
fn deep_decision_clone_before_move() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "clone_before_move")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains(".clone()"));
    assert_eq!(ps[0].error_code, "E0382");
}

#[test]
fn deep_decision_borrow_instead_of_move() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "borrow_instead_of_move")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("&x"));
}

#[test]
fn deep_decision_borrow_parameter() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "borrow_parameter")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("&String"));
}

#[test]
fn deep_decision_sequential_mutable_borrow() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "sequential_mutable_borrow")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("drop"));
}

#[test]
fn deep_decision_use_stdlib_method() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "use_stdlib_method")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("arr.swap"));
}

#[test]
fn deep_decision_reorder_borrow() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "reorder_borrow")
        .collect();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].error_code, "E0506");
}

#[test]
fn deep_decision_extend_lifetime() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "extend_lifetime")
        .collect();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].error_code, "E0597");
    assert!(ps[0].description.contains("outer scope"));
}

#[test]
fn deep_decision_return_owned() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "return_owned")
        .collect();
    assert_eq!(ps.len(), 2);
    let codes: HashSet<&str> = ps.iter().map(|p| p.error_code).collect();
    assert!(codes.contains("E0597"));
    assert!(codes.contains("E0515"));
}

#[test]
fn deep_decision_clone_return() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "clone_return")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains(".clone()"));
    assert_eq!(ps[0].error_code, "E0515");
}

#[test]
fn deep_decision_array_to_slice() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "array_to_slice")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("&[i32]"));
}

#[test]
fn deep_decision_bounds_checked_access() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "bounds_checked_access")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains(".get(i)"));
    assert!(ps[0].fix_diff.contains("unwrap_or"));
}

#[test]
fn deep_decision_safe_pointer_arithmetic() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "safe_pointer_arithmetic")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("wrapping_add"));
}

#[test]
fn deep_decision_malloc_to_box() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "malloc_to_box")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("Box::new"));
    assert!(ps[0].fix_diff.contains("malloc"));
}

#[test]
fn deep_decision_malloc_array_to_vec() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "malloc_array_to_vec")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("Vec::with_capacity"));
}

#[test]
fn deep_decision_arrow_to_dot() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "arrow_to_dot")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("p->field"));
    assert!(ps[0].fix_diff.contains("p.field"));
}

#[test]
fn deep_decision_nullable_to_option() {
    let ps: Vec<_> = get_bootstrap_patterns()
        .into_iter()
        .filter(|p| p.decision == "nullable_to_option")
        .collect();
    assert_eq!(ps.len(), 1);
    assert!(ps[0].fix_diff.contains("Option<Box<Node>>"));
}

// ============================================================================
// Section 4: fix_diff format validation
// ============================================================================

#[test]
fn deep_all_fix_diffs_nonempty() {
    for p in get_bootstrap_patterns() {
        assert!(!p.fix_diff.is_empty(), "Empty fix_diff for {}", p.decision);
    }
}

#[test]
fn deep_all_fix_diffs_have_minus_line() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.fix_diff.contains("- ") || p.fix_diff.starts_with('-'),
            "fix_diff missing minus line for {}: {}",
            p.decision,
            p.fix_diff
        );
    }
}

#[test]
fn deep_all_fix_diffs_have_plus_line() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.fix_diff.contains("+ ") || p.fix_diff.starts_with('+'),
            "fix_diff missing plus line for {}: {}",
            p.decision,
            p.fix_diff
        );
    }
}

#[test]
fn deep_all_fix_diffs_contain_newline() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.fix_diff.contains('\n'),
            "fix_diff should have newline for {}: {}",
            p.decision,
            p.fix_diff
        );
    }
}

// ============================================================================
// Section 5: description validation
// ============================================================================

#[test]
fn deep_all_descriptions_nonempty() {
    for p in get_bootstrap_patterns() {
        assert!(
            !p.description.is_empty(),
            "Empty description for {}",
            p.decision
        );
    }
}

#[test]
fn deep_all_descriptions_start_uppercase() {
    for p in get_bootstrap_patterns() {
        let first = p.description.chars().next().unwrap();
        assert!(
            first.is_uppercase(),
            "Description should start uppercase: '{}'",
            p.description
        );
    }
}

#[test]
fn deep_all_decisions_snake_case() {
    for p in get_bootstrap_patterns() {
        assert!(
            p.decision.chars().all(|c| c.is_lowercase() || c == '_'),
            "Decision not snake_case: '{}'",
            p.decision
        );
    }
}

// ============================================================================
// Section 6: specific pattern content tests (by index)
// ============================================================================

#[test]
fn deep_pattern_0_type_cast() {
    let p = &get_bootstrap_patterns()[0];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "type_coercion");
    assert!(p.fix_diff.contains("as i32"));
    assert!(p.description.contains("explicit type cast"));
}

#[test]
fn deep_pattern_1_mut_ptr_to_ref() {
    let p = &get_bootstrap_patterns()[1];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "pointer_to_reference");
    assert!(p.fix_diff.contains("*mut i32"));
    assert!(p.fix_diff.contains("&mut i32"));
    assert!(p.description.contains("mutable reference"));
}

#[test]
fn deep_pattern_2_const_ptr_to_ref() {
    let p = &get_bootstrap_patterns()[2];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "pointer_to_reference");
    assert!(p.fix_diff.contains("*const i32"));
    assert!(p.fix_diff.contains("&i32"));
    assert!(p.description.contains("immutable reference"));
}

#[test]
fn deep_pattern_3_mutable_swap_ref() {
    let p = &get_bootstrap_patterns()[3];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "mutable_reference");
    assert!(p.fix_diff.contains("swap"));
    assert!(p.fix_diff.contains("&mut x"));
}

#[test]
fn deep_pattern_4_exit_coercion() {
    let p = &get_bootstrap_patterns()[4];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "type_coercion");
    assert!(p.fix_diff.contains("std::process::exit"));
    assert!(p.fix_diff.contains("as i32"));
}

#[test]
fn deep_pattern_5_unsafe_write() {
    let p = &get_bootstrap_patterns()[5];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_deref");
    assert!(p.fix_diff.contains("*ptr = value"));
    assert!(p.fix_diff.contains("unsafe"));
}

#[test]
fn deep_pattern_6_unsafe_read() {
    let p = &get_bootstrap_patterns()[6];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_deref");
    assert!(p.fix_diff.contains("let x = *ptr"));
    assert!(p.fix_diff.contains("unsafe { *ptr }"));
}

#[test]
fn deep_pattern_7_unsafe_extern_fn() {
    let p = &get_bootstrap_patterns()[7];
    assert_eq!(p.error_code, "E0133");
    assert_eq!(p.decision, "unsafe_extern");
    assert!(p.fix_diff.contains("extern_fn()"));
    assert!(p.fix_diff.contains("unsafe { extern_fn(); }"));
}

#[test]
fn deep_pattern_8_clone_move() {
    let p = &get_bootstrap_patterns()[8];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "clone_before_move");
    assert!(p.fix_diff.contains("value.clone()"));
}

#[test]
fn deep_pattern_9_borrow_not_move() {
    let p = &get_bootstrap_patterns()[9];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "borrow_instead_of_move");
    assert!(p.fix_diff.contains("let y = &x"));
}

#[test]
fn deep_pattern_10_borrow_param() {
    let p = &get_bootstrap_patterns()[10];
    assert_eq!(p.error_code, "E0382");
    assert_eq!(p.decision, "borrow_parameter");
    assert!(p.fix_diff.contains("fn take(s: &String)"));
}

#[test]
fn deep_pattern_11_seq_mut_borrow() {
    let p = &get_bootstrap_patterns()[11];
    assert_eq!(p.error_code, "E0499");
    assert_eq!(p.decision, "sequential_mutable_borrow");
    assert!(p.fix_diff.contains("drop(a)"));
}

#[test]
fn deep_pattern_12_stdlib_swap() {
    let p = &get_bootstrap_patterns()[12];
    assert_eq!(p.error_code, "E0499");
    assert_eq!(p.decision, "use_stdlib_method");
    assert!(p.fix_diff.contains("arr.swap(i, j)"));
}

#[test]
fn deep_pattern_13_reorder() {
    let p = &get_bootstrap_patterns()[13];
    assert_eq!(p.error_code, "E0506");
    assert_eq!(p.decision, "reorder_borrow");
    assert!(p.fix_diff.contains("x = 5"));
}

#[test]
fn deep_pattern_14_extend_life() {
    let p = &get_bootstrap_patterns()[14];
    assert_eq!(p.error_code, "E0597");
    assert_eq!(p.decision, "extend_lifetime");
    assert!(p.description.contains("outer scope"));
}

#[test]
fn deep_pattern_15_return_owned_e0597() {
    let p = &get_bootstrap_patterns()[15];
    assert_eq!(p.error_code, "E0597");
    assert_eq!(p.decision, "return_owned");
    assert!(p.fix_diff.contains("-> i32"));
}

#[test]
fn deep_pattern_16_return_owned_e0515() {
    let p = &get_bootstrap_patterns()[16];
    assert_eq!(p.error_code, "E0515");
    assert_eq!(p.decision, "return_owned");
    assert!(p.fix_diff.contains("-> Vec<i32>"));
}

#[test]
fn deep_pattern_17_clone_return_local() {
    let p = &get_bootstrap_patterns()[17];
    assert_eq!(p.error_code, "E0515");
    assert_eq!(p.decision, "clone_return");
    assert!(p.fix_diff.contains("local.clone()"));
}

#[test]
fn deep_pattern_18_array_to_slice_conv() {
    let p = &get_bootstrap_patterns()[18];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "array_to_slice");
    assert!(p.fix_diff.contains("&[i32]"));
    assert!(p.description.contains("slice"));
}

#[test]
fn deep_pattern_19_bounds_check() {
    let p = &get_bootstrap_patterns()[19];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "bounds_checked_access");
    assert!(p.fix_diff.contains("arr.get(i).copied().unwrap_or(0)"));
}

#[test]
fn deep_pattern_20_safe_ptr_arith() {
    let p = &get_bootstrap_patterns()[20];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "safe_pointer_arithmetic");
    assert!(p.fix_diff.contains("ptr.wrapping_add(offset)"));
}

#[test]
fn deep_pattern_21_malloc_box() {
    let p = &get_bootstrap_patterns()[21];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "malloc_to_box");
    assert!(p.fix_diff.contains("Box::new(value)"));
    assert!(p.fix_diff.contains("malloc(size)"));
}

#[test]
fn deep_pattern_22_malloc_vec() {
    let p = &get_bootstrap_patterns()[22];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "malloc_array_to_vec");
    assert!(p.fix_diff.contains("Vec::with_capacity(n)"));
}

#[test]
fn deep_pattern_23_arrow_dot() {
    let p = &get_bootstrap_patterns()[23];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "arrow_to_dot");
    assert!(p.fix_diff.contains("p->field"));
    assert!(p.fix_diff.contains("p.field"));
}

#[test]
fn deep_pattern_24_nullable_option() {
    let p = &get_bootstrap_patterns()[24];
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "nullable_to_option");
    assert!(p.fix_diff.contains("Option<Box<Node>>"));
    assert!(p.fix_diff.contains("struct Node *next"));
}

// ============================================================================
// Section 7: BootstrapStats coverage
// ============================================================================

#[test]
fn deep_stats_from_patterns_total() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.total_patterns, 25);
}

#[test]
fn deep_stats_error_code_map_size() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.by_error_code.len(), 7);
}

#[test]
fn deep_stats_decision_map_size() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(stats.by_decision.len(), 21);
}

#[test]
fn deep_stats_error_e0308_is_12() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0308").unwrap(), 12);
}

#[test]
fn deep_stats_error_e0133_is_3() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0133").unwrap(), 3);
}

#[test]
fn deep_stats_error_e0382_is_3() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0382").unwrap(), 3);
}

#[test]
fn deep_stats_error_e0499_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0499").unwrap(), 2);
}

#[test]
fn deep_stats_error_e0506_is_1() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0506").unwrap(), 1);
}

#[test]
fn deep_stats_error_e0597_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0597").unwrap(), 2);
}

#[test]
fn deep_stats_error_e0515_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_error_code.get("E0515").unwrap(), 2);
}

#[test]
fn deep_stats_decision_type_coercion_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_decision.get("type_coercion").unwrap(), 2);
}

#[test]
fn deep_stats_decision_return_owned_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_decision.get("return_owned").unwrap(), 2);
}

#[test]
fn deep_stats_decision_pointer_to_reference_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_decision.get("pointer_to_reference").unwrap(), 2);
}

#[test]
fn deep_stats_decision_unsafe_deref_is_2() {
    let stats = BootstrapStats::from_patterns();
    assert_eq!(*stats.by_decision.get("unsafe_deref").unwrap(), 2);
}

#[test]
fn deep_stats_singleton_decisions_all_have_count_1() {
    let stats = BootstrapStats::from_patterns();
    let singletons = [
        "mutable_reference",
        "unsafe_extern",
        "clone_before_move",
        "borrow_instead_of_move",
        "borrow_parameter",
        "sequential_mutable_borrow",
        "use_stdlib_method",
        "reorder_borrow",
        "extend_lifetime",
        "clone_return",
        "array_to_slice",
        "bounds_checked_access",
        "safe_pointer_arithmetic",
        "malloc_to_box",
        "malloc_array_to_vec",
        "arrow_to_dot",
        "nullable_to_option",
    ];
    for name in singletons {
        assert_eq!(
            *stats.by_decision.get(name).unwrap(),
            1,
            "Decision {} should have count 1",
            name
        );
    }
}

// ============================================================================
// Section 8: BootstrapStats::to_string_pretty() coverage
// ============================================================================

#[test]
fn deep_stats_pretty_contains_header() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    assert!(pretty.contains("Bootstrap Patterns: 25"));
}

#[test]
fn deep_stats_pretty_contains_error_section() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    assert!(pretty.contains("By Error Code:"));
}

#[test]
fn deep_stats_pretty_contains_decision_section() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    assert!(pretty.contains("By Decision Type:"));
}

#[test]
fn deep_stats_pretty_error_codes_sorted() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    let pos_133 = pretty.find("E0133").unwrap();
    let pos_308 = pretty.find("E0308").unwrap();
    let pos_382 = pretty.find("E0382").unwrap();
    let pos_499 = pretty.find("E0499").unwrap();
    let pos_506 = pretty.find("E0506").unwrap();
    let pos_597 = pretty.find("E0597").unwrap();
    assert!(pos_133 < pos_308);
    assert!(pos_308 < pos_382);
    assert!(pos_382 < pos_499);
    assert!(pos_499 < pos_506);
    assert!(pos_506 < pos_597);
}

#[test]
fn deep_stats_pretty_has_e0308_12() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    assert!(pretty.contains("E0308: 12"));
}

#[test]
fn deep_stats_pretty_has_e0133_3() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    assert!(pretty.contains("E0133: 3"));
}

#[test]
fn deep_stats_pretty_decisions_descending() {
    let pretty = BootstrapStats::from_patterns().to_string_pretty();
    let dec_start = pretty.find("By Decision Type:").unwrap();
    let section = &pretty[dec_start..];
    // type_coercion (count 2) should appear before arrow_to_dot (count 1)
    let tc_pos = section.find("type_coercion").unwrap();
    let ad_pos = section.find("arrow_to_dot").unwrap();
    assert!(tc_pos < ad_pos);
}

// ============================================================================
// Section 9: BootstrapStats default + Debug + Clone
// ============================================================================

#[test]
fn deep_stats_default_zeroed() {
    let stats = BootstrapStats::default();
    assert_eq!(stats.total_patterns, 0);
    assert!(stats.by_error_code.is_empty());
    assert!(stats.by_decision.is_empty());
}

#[test]
fn deep_stats_debug_output() {
    let stats = BootstrapStats::from_patterns();
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("BootstrapStats"));
    assert!(dbg.contains("total_patterns"));
    assert!(dbg.contains("by_error_code"));
    assert!(dbg.contains("by_decision"));
}

#[test]
fn deep_pattern_debug_output() {
    let p = &get_bootstrap_patterns()[0];
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("BootstrapPattern"));
    assert!(dbg.contains("E0308"));
    assert!(dbg.contains("type_coercion"));
}

#[test]
fn deep_pattern_clone_preserves_fields() {
    let p = get_bootstrap_patterns()[0].clone();
    assert_eq!(p.error_code, "E0308");
    assert_eq!(p.decision, "type_coercion");
    assert!(!p.fix_diff.is_empty());
    assert!(!p.description.is_empty());
}

// ============================================================================
// Section 10: Cross-cutting validation
// ============================================================================

#[test]
fn deep_all_e0133_patterns_mention_unsafe_in_diff() {
    for p in get_bootstrap_patterns() {
        if p.error_code == "E0133" {
            assert!(
                p.fix_diff.contains("unsafe"),
                "E0133 pattern should have 'unsafe' in diff: {}",
                p.decision
            );
        }
    }
}

#[test]
fn deep_all_e0382_patterns_relate_to_ownership() {
    let ownership_decisions = [
        "clone_before_move",
        "borrow_instead_of_move",
        "borrow_parameter",
    ];
    for p in get_bootstrap_patterns() {
        if p.error_code == "E0382" {
            assert!(
                ownership_decisions.contains(&p.decision),
                "E0382 pattern should have ownership decision, got: {}",
                p.decision
            );
        }
    }
}

#[test]
fn deep_no_duplicate_patterns() {
    let patterns = get_bootstrap_patterns();
    for (i, a) in patterns.iter().enumerate() {
        for (j, b) in patterns.iter().enumerate() {
            if i != j {
                let same = a.error_code == b.error_code
                    && a.decision == b.decision
                    && a.fix_diff == b.fix_diff;
                assert!(
                    !same,
                    "Duplicate pattern found at indices {} and {}",
                    i, j
                );
            }
        }
    }
}

#[test]
fn deep_iterate_all_fields_for_coverage() {
    let patterns = get_bootstrap_patterns();
    let mut total_diff_len = 0;
    let mut total_desc_len = 0;
    for p in &patterns {
        total_diff_len += p.fix_diff.len();
        total_desc_len += p.description.len();
        // Touch every field to ensure coverage
        let _ = p.error_code.len();
        let _ = p.decision.len();
    }
    assert!(total_diff_len > 0);
    assert!(total_desc_len > 0);
}

#[test]
fn deep_stats_error_sum_matches_total() {
    let stats = BootstrapStats::from_patterns();
    let sum: usize = stats.by_error_code.values().sum();
    assert_eq!(sum, stats.total_patterns);
}

#[test]
fn deep_stats_decision_sum_matches_total() {
    let stats = BootstrapStats::from_patterns();
    let sum: usize = stats.by_decision.values().sum();
    assert_eq!(sum, stats.total_patterns);
}
