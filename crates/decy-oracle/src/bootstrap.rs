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
        assert!(store.len() > 0, "Bootstrapped store should have patterns");
    }

    #[cfg(feature = "citl")]
    #[test]
    fn test_bootstrapped_store_has_suggestions() {
        let store = create_bootstrapped_store().unwrap();

        // Should be able to get suggestions for E0308
        let suggestions = store.suggest_fix("E0308", &[], 5).unwrap();
        assert!(!suggestions.is_empty(), "Should have suggestions for E0308");
    }
}
