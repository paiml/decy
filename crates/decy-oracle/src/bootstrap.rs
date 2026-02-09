//! Bootstrap module for oracle cold start
//!
//! This module provides seed patterns for common C-to-Rust transpilation errors,
//! solving the cold start problem where the oracle has no patterns to learn from.
//!
//! # Toyota Way Principles
//!
//! - **Genchi Genbutsu** (現地現物): Patterns derived from real C→Rust transpilation errors
//! - **Yokoten** (横展): Cross-project pattern sharing from depyler ownership patterns
//! - **Jidoka** (自働化): Automated bootstrap when no patterns exist

#[cfg(feature = "citl")]
use entrenar::citl::{DecisionPatternStore, FixPattern};

#[cfg(feature = "citl")]
use crate::error::OracleError;

/// Bootstrap pattern definition
#[derive(Debug, Clone)]
pub struct BootstrapPattern {
    /// Error code (e.g., "E0308")
    pub error_code: &'static str,
    /// Fix diff showing the transformation
    pub fix_diff: &'static str,
    /// Decision context (e.g., "type_coercion", "unsafe_block")
    pub decision: &'static str,
    /// Human-readable description
    pub description: &'static str,
}

/// Get all bootstrap patterns for C→Rust transpilation
pub fn get_bootstrap_patterns() -> Vec<BootstrapPattern> {
    vec![
        // ============================================
        // E0308: Type Mismatch Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- let x: i32 = value;\n+ let x: i32 = value as i32;",
            decision: "type_coercion",
            description: "Add explicit type cast for integer conversion",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- fn foo(a: *mut i32)\n+ fn foo(a: &mut i32)",
            decision: "pointer_to_reference",
            description: "Convert raw pointer parameter to mutable reference",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- fn foo(a: *const i32)\n+ fn foo(a: &i32)",
            decision: "pointer_to_reference",
            description: "Convert raw pointer parameter to immutable reference",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- swap(&x, &y);\n+ swap(&mut x, &mut y);",
            decision: "mutable_reference",
            description: "Change immutable reference to mutable reference",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- exit(x);\n+ std::process::exit(x as i32);",
            decision: "type_coercion",
            description: "Cast to correct type for stdlib function",
        },

        // ============================================
        // E0133: Unsafe Block Required Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0133",
            fix_diff: "- *ptr = value;\n+ unsafe { *ptr = value; }",
            decision: "unsafe_deref",
            description: "Wrap pointer dereference in unsafe block",
        },
        BootstrapPattern {
            error_code: "E0133",
            fix_diff: "- let x = *ptr;\n+ let x = unsafe { *ptr };",
            decision: "unsafe_deref",
            description: "Wrap pointer read in unsafe block",
        },
        BootstrapPattern {
            error_code: "E0133",
            fix_diff: "- extern_fn();\n+ unsafe { extern_fn(); }",
            decision: "unsafe_extern",
            description: "Wrap extern function call in unsafe block",
        },

        // ============================================
        // E0382: Use After Move Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0382",
            fix_diff: "- process(value);\n- use(value);\n+ process(value.clone());\n+ use(value);",
            decision: "clone_before_move",
            description: "Clone value before move to allow subsequent use",
        },
        BootstrapPattern {
            error_code: "E0382",
            fix_diff: "- let y = x;\n- use(x);\n+ let y = &x;\n+ use(x);",
            decision: "borrow_instead_of_move",
            description: "Borrow instead of move to preserve ownership",
        },
        BootstrapPattern {
            error_code: "E0382",
            fix_diff: "- fn take(s: String)\n+ fn take(s: &String)",
            decision: "borrow_parameter",
            description: "Change function parameter to borrow instead of taking ownership",
        },

        // ============================================
        // E0499: Multiple Mutable Borrows Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0499",
            fix_diff: "- let a = &mut x;\n- let b = &mut x;\n+ let a = &mut x;\n+ drop(a);\n+ let b = &mut x;",
            decision: "sequential_mutable_borrow",
            description: "End first mutable borrow before starting second",
        },
        BootstrapPattern {
            error_code: "E0499",
            fix_diff: "- swap(&mut arr[i], &mut arr[j]);\n+ arr.swap(i, j);",
            decision: "use_stdlib_method",
            description: "Use stdlib method to avoid multiple mutable borrows",
        },

        // ============================================
        // E0506: Cannot Assign to Borrowed Value Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0506",
            fix_diff: "- let r = &x;\n- x = 5;\n- use(r);\n+ x = 5;\n+ let r = &x;\n+ use(r);",
            decision: "reorder_borrow",
            description: "Reorder borrow to occur after assignment",
        },

        // ============================================
        // E0597: Value Does Not Live Long Enough Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0597",
            fix_diff: "- let r;\n- {\n-     let x = 5;\n-     r = &x;\n- }\n+ let x = 5;\n+ let r = &x;",
            decision: "extend_lifetime",
            description: "Move value to outer scope to extend lifetime",
        },
        BootstrapPattern {
            error_code: "E0597",
            fix_diff: "- fn get_ref() -> &i32\n+ fn get_ref() -> i32",
            decision: "return_owned",
            description: "Return owned value instead of reference",
        },

        // ============================================
        // E0515: Cannot Return Reference to Local Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0515",
            fix_diff: "- fn create() -> &Vec<i32> {\n-     let v = vec![1,2,3];\n-     &v\n- }\n+ fn create() -> Vec<i32> {\n+     let v = vec![1,2,3];\n+     v\n+ }",
            decision: "return_owned",
            description: "Return owned value instead of reference to local",
        },
        BootstrapPattern {
            error_code: "E0515",
            fix_diff: "- return &local;\n+ return local.clone();",
            decision: "clone_return",
            description: "Clone local value to return owned copy",
        },

        // ============================================
        // C-Specific: Array/Pointer Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- fn process(arr: *const i32, len: usize)\n+ fn process(arr: &[i32])",
            decision: "array_to_slice",
            description: "Convert C array pointer to Rust slice",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- arr[i]\n+ arr.get(i).copied().unwrap_or(0)",
            decision: "bounds_checked_access",
            description: "Add bounds checking to array access",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- ptr + offset\n+ ptr.wrapping_add(offset)",
            decision: "safe_pointer_arithmetic",
            description: "Use safe pointer arithmetic method",
        },

        // ============================================
        // C-Specific: malloc/free Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- let ptr = malloc(size);\n+ let ptr = Box::new(value);",
            decision: "malloc_to_box",
            description: "Replace malloc with Box allocation",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- let arr = malloc(n * size);\n+ let arr = Vec::with_capacity(n);",
            decision: "malloc_array_to_vec",
            description: "Replace array malloc with Vec",
        },

        // ============================================
        // C-Specific: Struct Patterns
        // ============================================
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- p->field\n+ p.field",
            decision: "arrow_to_dot",
            description: "Replace C arrow operator with Rust dot operator",
        },
        BootstrapPattern {
            error_code: "E0308",
            fix_diff: "- struct Node *next;\n+ next: Option<Box<Node>>,",
            decision: "nullable_to_option",
            description: "Replace nullable pointer with Option<Box<T>>",
        },
    ]
}

/// Seed the oracle pattern store with bootstrap patterns
#[cfg(feature = "citl")]
pub fn seed_pattern_store(store: &mut DecisionPatternStore) -> Result<usize, OracleError> {
    let patterns = get_bootstrap_patterns();
    let mut count = 0;

    for bp in patterns {
        let pattern = FixPattern::new(bp.error_code, bp.fix_diff).with_decision(bp.decision);

        if store.index_fix(pattern).is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

/// Create a new pattern store with bootstrap patterns pre-loaded
#[cfg(feature = "citl")]
pub fn create_bootstrapped_store() -> Result<DecisionPatternStore, OracleError> {
    let mut store =
        DecisionPatternStore::new().map_err(|e| OracleError::PatternStoreError(e.to_string()))?;

    seed_pattern_store(&mut store)?;

    Ok(store)
}

/// Bootstrap statistics
#[derive(Debug, Default)]
pub struct BootstrapStats {
    /// Total patterns available
    pub total_patterns: usize,
    /// Patterns by error code
    pub by_error_code: std::collections::HashMap<String, usize>,
    /// Patterns by decision type
    pub by_decision: std::collections::HashMap<String, usize>,
}

impl BootstrapStats {
    /// Calculate statistics from bootstrap patterns
    pub fn from_patterns() -> Self {
        let patterns = get_bootstrap_patterns();
        let mut stats = Self {
            total_patterns: patterns.len(),
            ..Default::default()
        };

        for p in patterns {
            *stats
                .by_error_code
                .entry(p.error_code.to_string())
                .or_default() += 1;
            *stats.by_decision.entry(p.decision.to_string()).or_default() += 1;
        }

        stats
    }

    /// Format as human-readable string
    pub fn to_string_pretty(&self) -> String {
        let mut s = format!("Bootstrap Patterns: {}\n\n", self.total_patterns);

        s.push_str("By Error Code:\n");
        let mut codes: Vec<_> = self.by_error_code.iter().collect();
        codes.sort_by_key(|(k, _)| *k);
        for (code, count) in codes {
            s.push_str(&format!("  {}: {}\n", code, count));
        }

        s.push_str("\nBy Decision Type:\n");
        let mut decisions: Vec<_> = self.by_decision.iter().collect();
        decisions.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        for (decision, count) in decisions {
            s.push_str(&format!("  {}: {}\n", decision, count));
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_patterns_not_empty() {
        let patterns = get_bootstrap_patterns();
        assert!(
            !patterns.is_empty(),
            "Bootstrap patterns should not be empty"
        );
    }

    #[test]
    fn test_bootstrap_patterns_count() {
        let patterns = get_bootstrap_patterns();
        // Should have substantial coverage
        assert!(
            patterns.len() >= 20,
            "Should have at least 20 bootstrap patterns"
        );
    }

    #[test]
    fn test_all_patterns_have_valid_error_codes() {
        let patterns = get_bootstrap_patterns();
        for p in patterns {
            assert!(
                p.error_code.starts_with("E"),
                "Error code should start with E: {}",
                p.error_code
            );
            assert!(
                p.error_code.len() == 5,
                "Error code should be 5 chars (EXXXX): {}",
                p.error_code
            );
        }
    }

    #[test]
    fn test_all_patterns_have_fix_diffs() {
        let patterns = get_bootstrap_patterns();
        for p in patterns {
            assert!(
                !p.fix_diff.is_empty(),
                "Fix diff should not be empty for {}",
                p.error_code
            );
            assert!(
                p.fix_diff.contains('-') || p.fix_diff.contains('+'),
                "Fix diff should contain - or +: {}",
                p.fix_diff
            );
        }
    }

    #[test]
    fn test_all_patterns_have_decisions() {
        let patterns = get_bootstrap_patterns();
        for p in patterns {
            assert!(
                !p.decision.is_empty(),
                "Decision should not be empty for {}",
                p.error_code
            );
        }
    }

    #[test]
    fn test_all_patterns_have_descriptions() {
        let patterns = get_bootstrap_patterns();
        for p in patterns {
            assert!(
                !p.description.is_empty(),
                "Description should not be empty for {}",
                p.error_code
            );
        }
    }

    #[test]
    fn test_bootstrap_stats() {
        let stats = BootstrapStats::from_patterns();
        assert!(stats.total_patterns > 0);
        assert!(!stats.by_error_code.is_empty());
        assert!(!stats.by_decision.is_empty());
    }

    #[test]
    fn test_bootstrap_stats_has_common_error_codes() {
        let stats = BootstrapStats::from_patterns();
        // Should have patterns for key C→Rust errors
        assert!(
            stats.by_error_code.contains_key("E0308"),
            "Should have E0308 (type mismatch)"
        );
        assert!(
            stats.by_error_code.contains_key("E0133"),
            "Should have E0133 (unsafe)"
        );
        assert!(
            stats.by_error_code.contains_key("E0382"),
            "Should have E0382 (use after move)"
        );
    }

    #[test]
    fn test_bootstrap_stats_pretty_format() {
        let stats = BootstrapStats::from_patterns();
        let pretty = stats.to_string_pretty();
        assert!(pretty.contains("Bootstrap Patterns:"));
        assert!(pretty.contains("By Error Code:"));
        assert!(pretty.contains("By Decision Type:"));
    }

    #[cfg(feature = "citl")]
    #[test]
    fn test_seed_pattern_store() {
        let mut store = DecisionPatternStore::new().unwrap();
        let count = seed_pattern_store(&mut store).unwrap();
        assert!(count > 0, "Should seed at least some patterns");
        assert_eq!(
            count,
            store.len(),
            "Store should contain all seeded patterns"
        );
    }

    #[cfg(feature = "citl")]
    #[test]
    fn test_create_bootstrapped_store() {
        let store = create_bootstrapped_store().unwrap();
        assert!(!store.is_empty(), "Bootstrapped store should have patterns");
    }

    #[cfg(feature = "citl")]
    #[test]
    fn test_bootstrapped_store_has_suggestions() {
        let store = create_bootstrapped_store().unwrap();

        // Should be able to get suggestions for E0308
        let suggestions = store.suggest_fix("E0308", &[], 5).unwrap();
        assert!(!suggestions.is_empty(), "Should have suggestions for E0308");
    }

    // ========================================================================
    // Coverage tests: exercise every pattern's fields for line coverage
    // ========================================================================

    #[test]
    fn test_iterate_all_patterns_read_all_fields() {
        let patterns = get_bootstrap_patterns();
        let mut error_codes = Vec::new();
        let mut fix_diffs = Vec::new();
        let mut decisions = Vec::new();
        let mut descriptions = Vec::new();

        for p in &patterns {
            error_codes.push(p.error_code.to_string());
            fix_diffs.push(p.fix_diff.to_string());
            decisions.push(p.decision.to_string());
            descriptions.push(p.description.to_string());
        }

        assert_eq!(error_codes.len(), patterns.len());
        assert_eq!(fix_diffs.len(), patterns.len());
        assert_eq!(decisions.len(), patterns.len());
        assert_eq!(descriptions.len(), patterns.len());

        // Verify no field is empty across the entire collection
        assert!(error_codes.iter().all(|s| !s.is_empty()));
        assert!(fix_diffs.iter().all(|s| !s.is_empty()));
        assert!(decisions.iter().all(|s| !s.is_empty()));
        assert!(descriptions.iter().all(|s| !s.is_empty()));
    }

    #[test]
    fn test_exact_pattern_count() {
        let patterns = get_bootstrap_patterns();
        assert_eq!(patterns.len(), 25, "Should have exactly 25 bootstrap patterns");
    }

    // ========================================================================
    // Error code distribution tests
    // ========================================================================

    #[test]
    fn test_e0308_type_mismatch_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0308: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0308").collect();
        assert_eq!(
            e0308.len(),
            12,
            "E0308 should have 12 patterns (5 type mismatch + 3 array/pointer + 2 malloc + 2 struct)"
        );
        // Verify all E0308 patterns have type-related content
        for p in &e0308 {
            assert!(
                !p.fix_diff.is_empty() && !p.decision.is_empty(),
                "E0308 pattern missing content: {}",
                p.description
            );
        }
    }

    #[test]
    fn test_e0133_unsafe_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0133: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0133").collect();
        assert_eq!(e0133.len(), 3, "E0133 should have 3 unsafe patterns");
        for p in &e0133 {
            assert!(
                p.fix_diff.contains("unsafe"),
                "E0133 fix_diff should mention unsafe: {}",
                p.description
            );
        }
    }

    #[test]
    fn test_e0382_use_after_move_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0382: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0382").collect();
        assert_eq!(e0382.len(), 3, "E0382 should have 3 use-after-move patterns");
    }

    #[test]
    fn test_e0499_multiple_mutable_borrow_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0499: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0499").collect();
        assert_eq!(e0499.len(), 2, "E0499 should have 2 multiple mutable borrow patterns");
    }

    #[test]
    fn test_e0506_assign_to_borrowed_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0506: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0506").collect();
        assert_eq!(
            e0506.len(),
            1,
            "E0506 should have 1 cannot-assign-to-borrowed pattern"
        );
        assert_eq!(e0506[0].decision, "reorder_borrow");
        assert!(e0506[0].description.contains("Reorder"));
    }

    #[test]
    fn test_e0597_lifetime_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0597: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0597").collect();
        assert_eq!(e0597.len(), 2, "E0597 should have 2 lifetime patterns");
    }

    #[test]
    fn test_e0515_return_reference_to_local_patterns() {
        let patterns = get_bootstrap_patterns();
        let e0515: Vec<_> = patterns.iter().filter(|p| p.error_code == "E0515").collect();
        assert_eq!(
            e0515.len(),
            2,
            "E0515 should have 2 return-reference-to-local patterns"
        );
    }

    #[test]
    fn test_error_code_counts_sum_to_total() {
        let patterns = get_bootstrap_patterns();
        let total = patterns.len();
        let e0308 = patterns.iter().filter(|p| p.error_code == "E0308").count();
        let e0133 = patterns.iter().filter(|p| p.error_code == "E0133").count();
        let e0382 = patterns.iter().filter(|p| p.error_code == "E0382").count();
        let e0499 = patterns.iter().filter(|p| p.error_code == "E0499").count();
        let e0506 = patterns.iter().filter(|p| p.error_code == "E0506").count();
        let e0597 = patterns.iter().filter(|p| p.error_code == "E0597").count();
        let e0515 = patterns.iter().filter(|p| p.error_code == "E0515").count();
        assert_eq!(
            e0308 + e0133 + e0382 + e0499 + e0506 + e0597 + e0515,
            total,
            "All error codes should account for all patterns"
        );
    }

    // ========================================================================
    // Decision type distribution tests
    // ========================================================================

    #[test]
    fn test_decision_type_coercion_patterns() {
        let patterns = get_bootstrap_patterns();
        let coercion: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "type_coercion")
            .collect();
        assert_eq!(coercion.len(), 2, "type_coercion should have 2 patterns");
        assert!(coercion.iter().all(|p| p.error_code == "E0308"));
    }

    #[test]
    fn test_decision_pointer_to_reference_patterns() {
        let patterns = get_bootstrap_patterns();
        let ptr_ref: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "pointer_to_reference")
            .collect();
        assert_eq!(ptr_ref.len(), 2, "pointer_to_reference should have 2 patterns");
        // One for *mut, one for *const
        assert!(ptr_ref.iter().any(|p| p.fix_diff.contains("*mut")));
        assert!(ptr_ref.iter().any(|p| p.fix_diff.contains("*const")));
    }

    #[test]
    fn test_decision_mutable_reference_pattern() {
        let patterns = get_bootstrap_patterns();
        let mut_ref: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "mutable_reference")
            .collect();
        assert_eq!(mut_ref.len(), 1, "mutable_reference should have 1 pattern");
        assert!(mut_ref[0].fix_diff.contains("&mut"));
    }

    #[test]
    fn test_decision_unsafe_deref_patterns() {
        let patterns = get_bootstrap_patterns();
        let deref: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "unsafe_deref")
            .collect();
        assert_eq!(deref.len(), 2, "unsafe_deref should have 2 patterns");
        for p in &deref {
            assert!(p.fix_diff.contains("unsafe"));
            assert!(p.error_code == "E0133");
        }
    }

    #[test]
    fn test_decision_unsafe_extern_pattern() {
        let patterns = get_bootstrap_patterns();
        let ext: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "unsafe_extern")
            .collect();
        assert_eq!(ext.len(), 1, "unsafe_extern should have 1 pattern");
        assert!(ext[0].fix_diff.contains("extern_fn"));
        assert!(ext[0].fix_diff.contains("unsafe"));
    }

    #[test]
    fn test_decision_clone_before_move_pattern() {
        let patterns = get_bootstrap_patterns();
        let clone_move: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "clone_before_move")
            .collect();
        assert_eq!(clone_move.len(), 1, "clone_before_move should have 1 pattern");
        assert!(clone_move[0].fix_diff.contains(".clone()"));
        assert_eq!(clone_move[0].error_code, "E0382");
    }

    #[test]
    fn test_decision_borrow_instead_of_move_pattern() {
        let patterns = get_bootstrap_patterns();
        let borrow: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "borrow_instead_of_move")
            .collect();
        assert_eq!(borrow.len(), 1, "borrow_instead_of_move should have 1 pattern");
        assert!(borrow[0].fix_diff.contains("&x"));
    }

    #[test]
    fn test_decision_borrow_parameter_pattern() {
        let patterns = get_bootstrap_patterns();
        let borrow_param: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "borrow_parameter")
            .collect();
        assert_eq!(borrow_param.len(), 1, "borrow_parameter should have 1 pattern");
        assert!(borrow_param[0].fix_diff.contains("&String"));
    }

    #[test]
    fn test_decision_sequential_mutable_borrow_pattern() {
        let patterns = get_bootstrap_patterns();
        let seq_mut: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "sequential_mutable_borrow")
            .collect();
        assert_eq!(seq_mut.len(), 1, "sequential_mutable_borrow should have 1 pattern");
        assert!(seq_mut[0].fix_diff.contains("drop"));
    }

    #[test]
    fn test_decision_use_stdlib_method_pattern() {
        let patterns = get_bootstrap_patterns();
        let stdlib: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "use_stdlib_method")
            .collect();
        assert_eq!(stdlib.len(), 1, "use_stdlib_method should have 1 pattern");
        assert!(stdlib[0].fix_diff.contains("arr.swap"));
    }

    #[test]
    fn test_decision_reorder_borrow_pattern() {
        let patterns = get_bootstrap_patterns();
        let reorder: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "reorder_borrow")
            .collect();
        assert_eq!(reorder.len(), 1, "reorder_borrow should have 1 pattern");
        assert_eq!(reorder[0].error_code, "E0506");
    }

    #[test]
    fn test_decision_extend_lifetime_pattern() {
        let patterns = get_bootstrap_patterns();
        let extend: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "extend_lifetime")
            .collect();
        assert_eq!(extend.len(), 1, "extend_lifetime should have 1 pattern");
        assert_eq!(extend[0].error_code, "E0597");
        assert!(extend[0].description.contains("outer scope"));
    }

    #[test]
    fn test_decision_return_owned_patterns() {
        let patterns = get_bootstrap_patterns();
        let owned: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "return_owned")
            .collect();
        assert_eq!(owned.len(), 2, "return_owned should have 2 patterns (E0597 + E0515)");
        let error_codes: Vec<_> = owned.iter().map(|p| p.error_code).collect();
        assert!(error_codes.contains(&"E0597"));
        assert!(error_codes.contains(&"E0515"));
    }

    #[test]
    fn test_decision_clone_return_pattern() {
        let patterns = get_bootstrap_patterns();
        let clone_ret: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "clone_return")
            .collect();
        assert_eq!(clone_ret.len(), 1, "clone_return should have 1 pattern");
        assert!(clone_ret[0].fix_diff.contains(".clone()"));
        assert_eq!(clone_ret[0].error_code, "E0515");
    }

    #[test]
    fn test_decision_array_to_slice_pattern() {
        let patterns = get_bootstrap_patterns();
        let slice: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "array_to_slice")
            .collect();
        assert_eq!(slice.len(), 1, "array_to_slice should have 1 pattern");
        assert!(slice[0].fix_diff.contains("&[i32]"));
        assert!(slice[0].description.contains("slice"));
    }

    #[test]
    fn test_decision_bounds_checked_access_pattern() {
        let patterns = get_bootstrap_patterns();
        let bounds: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "bounds_checked_access")
            .collect();
        assert_eq!(bounds.len(), 1, "bounds_checked_access should have 1 pattern");
        assert!(bounds[0].fix_diff.contains(".get(i)"));
        assert!(bounds[0].fix_diff.contains("unwrap_or"));
    }

    #[test]
    fn test_decision_safe_pointer_arithmetic_pattern() {
        let patterns = get_bootstrap_patterns();
        let arith: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "safe_pointer_arithmetic")
            .collect();
        assert_eq!(arith.len(), 1, "safe_pointer_arithmetic should have 1 pattern");
        assert!(arith[0].fix_diff.contains("wrapping_add"));
    }

    #[test]
    fn test_decision_malloc_to_box_pattern() {
        let patterns = get_bootstrap_patterns();
        let malloc_box: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "malloc_to_box")
            .collect();
        assert_eq!(malloc_box.len(), 1, "malloc_to_box should have 1 pattern");
        assert!(
            malloc_box[0].fix_diff.contains("Box::new"),
            "malloc_to_box pattern should contain Box::new"
        );
        assert!(malloc_box[0].fix_diff.contains("malloc"));
    }

    #[test]
    fn test_decision_malloc_array_to_vec_pattern() {
        let patterns = get_bootstrap_patterns();
        let malloc_vec: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "malloc_array_to_vec")
            .collect();
        assert_eq!(malloc_vec.len(), 1, "malloc_array_to_vec should have 1 pattern");
        assert!(malloc_vec[0].fix_diff.contains("Vec::with_capacity"));
    }

    #[test]
    fn test_decision_arrow_to_dot_pattern() {
        let patterns = get_bootstrap_patterns();
        let arrow: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "arrow_to_dot")
            .collect();
        assert_eq!(arrow.len(), 1, "arrow_to_dot should have 1 pattern");
        assert!(arrow[0].fix_diff.contains("p->field"));
        assert!(arrow[0].fix_diff.contains("p.field"));
    }

    #[test]
    fn test_decision_nullable_to_option_pattern() {
        let patterns = get_bootstrap_patterns();
        let nullable: Vec<_> = patterns
            .iter()
            .filter(|p| p.decision == "nullable_to_option")
            .collect();
        assert_eq!(nullable.len(), 1, "nullable_to_option should have 1 pattern");
        assert!(nullable[0].fix_diff.contains("Option<Box<Node>>"));
    }

    #[test]
    fn test_all_decision_types_covered() {
        let patterns = get_bootstrap_patterns();
        let mut decisions: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for p in &patterns {
            decisions.insert(p.decision);
        }
        let expected_decisions = vec![
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
        assert_eq!(
            decisions.len(),
            expected_decisions.len(),
            "Number of unique decision types should match expected"
        );
        for d in &expected_decisions {
            assert!(decisions.contains(d), "Missing decision type: {}", d);
        }
    }

    // ========================================================================
    // Specific pattern content validation (ordered by vec position)
    // ========================================================================

    #[test]
    fn test_pattern_0_type_coercion_cast() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[0];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "type_coercion");
        assert!(p.fix_diff.contains("as i32"));
        assert!(p.description.contains("explicit type cast"));
    }

    #[test]
    fn test_pattern_1_mut_pointer_to_ref() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[1];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "pointer_to_reference");
        assert!(p.fix_diff.contains("*mut i32"));
        assert!(p.fix_diff.contains("&mut i32"));
    }

    #[test]
    fn test_pattern_2_const_pointer_to_ref() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[2];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "pointer_to_reference");
        assert!(p.fix_diff.contains("*const i32"));
        assert!(p.fix_diff.contains("&i32"));
    }

    #[test]
    fn test_pattern_3_mutable_reference() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[3];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "mutable_reference");
        assert!(p.fix_diff.contains("swap"));
        assert!(p.fix_diff.contains("&mut x"));
    }

    #[test]
    fn test_pattern_4_exit_cast() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[4];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "type_coercion");
        assert!(p.fix_diff.contains("std::process::exit"));
    }

    #[test]
    fn test_pattern_5_unsafe_deref_write() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[5];
        assert_eq!(p.error_code, "E0133");
        assert_eq!(p.decision, "unsafe_deref");
        assert!(p.fix_diff.contains("*ptr = value"));
        assert!(p.description.contains("dereference"));
    }

    #[test]
    fn test_pattern_6_unsafe_deref_read() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[6];
        assert_eq!(p.error_code, "E0133");
        assert_eq!(p.decision, "unsafe_deref");
        assert!(p.fix_diff.contains("let x = *ptr"));
        assert!(p.description.contains("pointer read"));
    }

    #[test]
    fn test_pattern_7_unsafe_extern() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[7];
        assert_eq!(p.error_code, "E0133");
        assert_eq!(p.decision, "unsafe_extern");
        assert!(p.fix_diff.contains("extern_fn()"));
    }

    #[test]
    fn test_pattern_8_clone_before_move() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[8];
        assert_eq!(p.error_code, "E0382");
        assert_eq!(p.decision, "clone_before_move");
        assert!(p.fix_diff.contains("value.clone()"));
    }

    #[test]
    fn test_pattern_9_borrow_instead_of_move() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[9];
        assert_eq!(p.error_code, "E0382");
        assert_eq!(p.decision, "borrow_instead_of_move");
        assert!(p.fix_diff.contains("let y = &x"));
    }

    #[test]
    fn test_pattern_10_borrow_parameter() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[10];
        assert_eq!(p.error_code, "E0382");
        assert_eq!(p.decision, "borrow_parameter");
        assert!(p.fix_diff.contains("fn take(s: &String)"));
    }

    #[test]
    fn test_pattern_11_sequential_mutable_borrow() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[11];
        assert_eq!(p.error_code, "E0499");
        assert_eq!(p.decision, "sequential_mutable_borrow");
        assert!(p.fix_diff.contains("drop(a)"));
    }

    #[test]
    fn test_pattern_12_use_stdlib_swap() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[12];
        assert_eq!(p.error_code, "E0499");
        assert_eq!(p.decision, "use_stdlib_method");
        assert!(p.fix_diff.contains("arr.swap(i, j)"));
    }

    #[test]
    fn test_pattern_13_reorder_borrow() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[13];
        assert_eq!(p.error_code, "E0506");
        assert_eq!(p.decision, "reorder_borrow");
        assert!(p.fix_diff.contains("x = 5"));
    }

    #[test]
    fn test_pattern_14_extend_lifetime() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[14];
        assert_eq!(p.error_code, "E0597");
        assert_eq!(p.decision, "extend_lifetime");
        assert!(p.description.contains("outer scope"));
    }

    #[test]
    fn test_pattern_15_return_owned_e0597() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[15];
        assert_eq!(p.error_code, "E0597");
        assert_eq!(p.decision, "return_owned");
        assert!(p.fix_diff.contains("-> i32"));
    }

    #[test]
    fn test_pattern_16_return_owned_e0515() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[16];
        assert_eq!(p.error_code, "E0515");
        assert_eq!(p.decision, "return_owned");
        assert!(p.fix_diff.contains("-> Vec<i32>"));
    }

    #[test]
    fn test_pattern_17_clone_return() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[17];
        assert_eq!(p.error_code, "E0515");
        assert_eq!(p.decision, "clone_return");
        assert!(p.fix_diff.contains("local.clone()"));
    }

    #[test]
    fn test_pattern_18_array_to_slice() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[18];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "array_to_slice");
        assert!(p.fix_diff.contains("&[i32]"));
    }

    #[test]
    fn test_pattern_19_bounds_checked_access() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[19];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "bounds_checked_access");
        assert!(p.fix_diff.contains("arr.get(i).copied().unwrap_or(0)"));
    }

    #[test]
    fn test_pattern_20_safe_pointer_arithmetic() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[20];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "safe_pointer_arithmetic");
        assert!(p.fix_diff.contains("ptr.wrapping_add(offset)"));
    }

    #[test]
    fn test_pattern_21_malloc_to_box() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[21];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "malloc_to_box");
        assert!(p.fix_diff.contains("Box::new(value)"));
    }

    #[test]
    fn test_pattern_22_malloc_array_to_vec() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[22];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "malloc_array_to_vec");
        assert!(p.fix_diff.contains("Vec::with_capacity(n)"));
    }

    #[test]
    fn test_pattern_23_arrow_to_dot() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[23];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "arrow_to_dot");
        assert!(p.fix_diff.contains("p->field"));
        assert!(p.fix_diff.contains("p.field"));
    }

    #[test]
    fn test_pattern_24_nullable_to_option() {
        let patterns = get_bootstrap_patterns();
        let p = &patterns[24];
        assert_eq!(p.error_code, "E0308");
        assert_eq!(p.decision, "nullable_to_option");
        assert!(p.fix_diff.contains("Option<Box<Node>>"));
    }

    // ========================================================================
    // BootstrapStats comprehensive tests
    // ========================================================================

    #[test]
    fn test_bootstrap_stats_total_matches_patterns() {
        let patterns = get_bootstrap_patterns();
        let stats = BootstrapStats::from_patterns();
        assert_eq!(stats.total_patterns, patterns.len());
    }

    #[test]
    fn test_bootstrap_stats_error_code_counts() {
        let stats = BootstrapStats::from_patterns();
        assert_eq!(stats.by_error_code.get("E0308"), Some(&12));
        assert_eq!(stats.by_error_code.get("E0133"), Some(&3));
        assert_eq!(stats.by_error_code.get("E0382"), Some(&3));
        assert_eq!(stats.by_error_code.get("E0499"), Some(&2));
        assert_eq!(stats.by_error_code.get("E0506"), Some(&1));
        assert_eq!(stats.by_error_code.get("E0597"), Some(&2));
        assert_eq!(stats.by_error_code.get("E0515"), Some(&2));
    }

    #[test]
    fn test_bootstrap_stats_error_code_count_is_7() {
        let stats = BootstrapStats::from_patterns();
        assert_eq!(
            stats.by_error_code.len(),
            7,
            "Should have exactly 7 distinct error codes"
        );
    }

    #[test]
    fn test_bootstrap_stats_decision_counts() {
        let stats = BootstrapStats::from_patterns();
        assert_eq!(stats.by_decision.get("type_coercion"), Some(&2));
        assert_eq!(stats.by_decision.get("pointer_to_reference"), Some(&2));
        assert_eq!(stats.by_decision.get("mutable_reference"), Some(&1));
        assert_eq!(stats.by_decision.get("unsafe_deref"), Some(&2));
        assert_eq!(stats.by_decision.get("unsafe_extern"), Some(&1));
        assert_eq!(stats.by_decision.get("clone_before_move"), Some(&1));
        assert_eq!(stats.by_decision.get("borrow_instead_of_move"), Some(&1));
        assert_eq!(stats.by_decision.get("borrow_parameter"), Some(&1));
        assert_eq!(stats.by_decision.get("sequential_mutable_borrow"), Some(&1));
        assert_eq!(stats.by_decision.get("use_stdlib_method"), Some(&1));
        assert_eq!(stats.by_decision.get("reorder_borrow"), Some(&1));
        assert_eq!(stats.by_decision.get("extend_lifetime"), Some(&1));
        assert_eq!(stats.by_decision.get("return_owned"), Some(&2));
        assert_eq!(stats.by_decision.get("clone_return"), Some(&1));
        assert_eq!(stats.by_decision.get("array_to_slice"), Some(&1));
        assert_eq!(stats.by_decision.get("bounds_checked_access"), Some(&1));
        assert_eq!(stats.by_decision.get("safe_pointer_arithmetic"), Some(&1));
        assert_eq!(stats.by_decision.get("malloc_to_box"), Some(&1));
        assert_eq!(stats.by_decision.get("malloc_array_to_vec"), Some(&1));
        assert_eq!(stats.by_decision.get("arrow_to_dot"), Some(&1));
        assert_eq!(stats.by_decision.get("nullable_to_option"), Some(&1));
    }

    #[test]
    fn test_bootstrap_stats_decision_count_is_21() {
        let stats = BootstrapStats::from_patterns();
        assert_eq!(
            stats.by_decision.len(),
            21,
            "Should have exactly 21 distinct decision types"
        );
    }

    #[test]
    fn test_bootstrap_stats_pretty_contains_all_error_codes() {
        let stats = BootstrapStats::from_patterns();
        let pretty = stats.to_string_pretty();
        assert!(pretty.contains("E0308"));
        assert!(pretty.contains("E0133"));
        assert!(pretty.contains("E0382"));
        assert!(pretty.contains("E0499"));
        assert!(pretty.contains("E0506"));
        assert!(pretty.contains("E0597"));
        assert!(pretty.contains("E0515"));
    }

    #[test]
    fn test_bootstrap_stats_pretty_contains_counts() {
        let stats = BootstrapStats::from_patterns();
        let pretty = stats.to_string_pretty();
        // E0308 has 12 patterns
        assert!(pretty.contains("E0308: 12"), "Pretty format should show E0308: 12");
    }

    #[test]
    fn test_bootstrap_stats_pretty_sorted_error_codes() {
        let stats = BootstrapStats::from_patterns();
        let pretty = stats.to_string_pretty();
        // Error codes should appear in sorted order
        let e0133_pos = pretty.find("E0133").expect("E0133 should be in output");
        let e0308_pos = pretty.find("E0308").expect("E0308 should be in output");
        let e0382_pos = pretty.find("E0382").expect("E0382 should be in output");
        let e0499_pos = pretty.find("E0499").expect("E0499 should be in output");
        let e0506_pos = pretty.find("E0506").expect("E0506 should be in output");
        assert!(e0133_pos < e0308_pos, "E0133 should appear before E0308");
        assert!(e0308_pos < e0382_pos, "E0308 should appear before E0382");
        assert!(e0382_pos < e0499_pos, "E0382 should appear before E0499");
        assert!(e0499_pos < e0506_pos, "E0499 should appear before E0506");
    }

    #[test]
    fn test_bootstrap_stats_pretty_decisions_sorted_by_count_descending() {
        let stats = BootstrapStats::from_patterns();
        let pretty = stats.to_string_pretty();
        // Decisions with count 2 should appear before decisions with count 1
        // type_coercion has count 2, arrow_to_dot has count 1
        let decision_section_start = pretty
            .find("By Decision Type:")
            .expect("Should have decision section");
        let type_coercion_pos = pretty[decision_section_start..]
            .find("type_coercion")
            .expect("type_coercion should be in output");
        let arrow_to_dot_pos = pretty[decision_section_start..]
            .find("arrow_to_dot")
            .expect("arrow_to_dot should be in output");
        assert!(
            type_coercion_pos < arrow_to_dot_pos,
            "type_coercion (count 2) should appear before arrow_to_dot (count 1)"
        );
    }

    #[test]
    fn test_bootstrap_stats_default() {
        let stats = BootstrapStats::default();
        assert_eq!(stats.total_patterns, 0);
        assert!(stats.by_error_code.is_empty());
        assert!(stats.by_decision.is_empty());
    }

    #[test]
    fn test_bootstrap_pattern_debug_impl() {
        let patterns = get_bootstrap_patterns();
        for p in &patterns {
            let debug_str = format!("{:?}", p);
            assert!(debug_str.contains("BootstrapPattern"));
            assert!(debug_str.contains(p.error_code));
            assert!(debug_str.contains(p.decision));
        }
    }

    #[test]
    fn test_bootstrap_pattern_clone_impl() {
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
    fn test_bootstrap_stats_debug_impl() {
        let stats = BootstrapStats::from_patterns();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("BootstrapStats"));
        assert!(debug_str.contains("total_patterns"));
    }

    // ========================================================================
    // Fix diff content validation (ensures diff format correctness)
    // ========================================================================

    #[test]
    fn test_all_fix_diffs_have_minus_and_plus_lines() {
        let patterns = get_bootstrap_patterns();
        for p in &patterns {
            assert!(
                p.fix_diff.contains('-'),
                "Pattern '{}' fix_diff missing '-' line: {}",
                p.decision,
                p.fix_diff
            );
            assert!(
                p.fix_diff.contains('+'),
                "Pattern '{}' fix_diff missing '+' line: {}",
                p.decision,
                p.fix_diff
            );
        }
    }

    #[test]
    fn test_all_descriptions_start_with_capital() {
        let patterns = get_bootstrap_patterns();
        for p in &patterns {
            assert!(
                p.description.starts_with(|c: char| c.is_uppercase()),
                "Description should start with capital letter: '{}'",
                p.description
            );
        }
    }

    #[test]
    fn test_all_decisions_are_snake_case() {
        let patterns = get_bootstrap_patterns();
        for p in &patterns {
            assert!(
                p.decision.chars().all(|c| c.is_lowercase() || c == '_'),
                "Decision should be snake_case: '{}'",
                p.decision
            );
        }
    }

    #[test]
    fn test_all_error_codes_are_numeric_after_e() {
        let patterns = get_bootstrap_patterns();
        for p in &patterns {
            let code_num = &p.error_code[1..];
            assert!(
                code_num.chars().all(|c| c.is_ascii_digit()),
                "Error code after E should be numeric: '{}'",
                p.error_code
            );
        }
    }
}
