//! Coverage tests for default_pattern_library() in ml_features.rs
//!
//! Exercises every pattern construction line in the function by
//! verifying each of the 18 patterns' fields: id, error_kind,
//! description, severity, curriculum_level, c_pattern, rust_error,
//! and suggested_fix.

use crate::ml_features::*;

// ============================================================================
// CORE: Verify exact count and non-emptiness
// ============================================================================

#[test]
fn default_library_returns_exactly_17_patterns() {
    let library = default_pattern_library();
    assert_eq!(library.len(), 17, "default_pattern_library should produce 17 patterns");
}

#[test]
fn default_library_is_not_empty() {
    let library = default_pattern_library();
    assert!(!library.is_empty());
}

// ============================================================================
// LEVEL 1: Basic Ownership Patterns
// ============================================================================

#[test]
fn level1_malloc_to_box_pattern() {
    let library = default_pattern_library();
    let p = library.get("malloc-to-box").expect("malloc-to-box missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::PointerMisclassification);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 1);
    assert_eq!(p.description(), "Single allocation with malloc should use Box<T>");
    assert_eq!(p.c_pattern(), Some("void *ptr = malloc(sizeof(T))"));
    assert_eq!(p.rust_error(), Some("E0308"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Replace raw pointer with Box");
    assert!(fix.code_template().contains("Box::new"));
    assert_eq!(p.occurrence_count(), 0);
}

#[test]
fn level1_array_to_vec_pattern() {
    let library = default_pattern_library();
    let p = library.get("array-to-vec").expect("array-to-vec missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::PointerMisclassification);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 1);
    assert_eq!(p.description(), "Dynamic array allocation should use Vec<T>");
    assert_eq!(p.c_pattern(), Some("T *arr = malloc(n * sizeof(T))"));
    assert_eq!(p.rust_error(), Some("E0308"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Replace pointer array with Vec");
    assert!(fix.code_template().contains("Vec::with_capacity"));
}

#[test]
fn level1_const_to_immut_ref_pattern() {
    let library = default_pattern_library();
    let p = library.get("const-to-immut-ref").expect("const-to-immut-ref missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::MutabilityMismatch);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 1);
    assert_eq!(p.description(), "Const pointer should be immutable reference");
    assert_eq!(p.c_pattern(), Some("const T *ptr"));
    assert_eq!(p.rust_error(), Some("E0596"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Use immutable reference");
    assert!(fix.code_template().contains("&T"));
}

// ============================================================================
// LEVEL 2: Lifetime Patterns
// ============================================================================

#[test]
fn level2_missing_lifetime_pattern() {
    let library = default_pattern_library();
    let p = library.get("missing-lifetime").expect("missing-lifetime missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::LifetimeInferenceGap);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 2);
    assert_eq!(
        p.description(),
        "Function returning reference needs lifetime annotation"
    );
    assert_eq!(
        p.c_pattern(),
        Some("T* get_field(Struct *s) { return &s->field; }")
    );
    assert_eq!(p.rust_error(), Some("E0106"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Add lifetime parameter");
    assert!(fix.code_template().contains("<'a>"));
}

#[test]
fn level2_struct_lifetime_pattern() {
    let library = default_pattern_library();
    let p = library.get("struct-lifetime").expect("struct-lifetime missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::LifetimeInferenceGap);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 2);
    assert_eq!(
        p.description(),
        "Struct containing reference needs lifetime parameter"
    );
    assert_eq!(p.c_pattern(), Some("struct View { T *data; }"));
    assert_eq!(p.rust_error(), Some("E0106"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Add lifetime to struct");
    assert!(fix.code_template().contains("View<'a>"));
}

#[test]
fn level2_array_param_to_slice_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("array-param-to-slice")
        .expect("array-param-to-slice missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::ArraySliceMismatch);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 2);
    assert_eq!(p.description(), "Array parameter should be slice reference");
    assert_eq!(
        p.c_pattern(),
        Some("void process(int arr[], size_t len)")
    );
    assert_eq!(p.rust_error(), Some("E0308"));
    let fix = p.suggested_fix().expect("should have fix");
    assert_eq!(fix.description(), "Use slice parameter");
    assert!(fix.code_template().contains("&[i32]"));
}

// ============================================================================
// LEVEL 3: Borrow Checker Patterns
// ============================================================================

#[test]
fn level3_mutable_aliasing_pattern() {
    let library = default_pattern_library();
    let p = library.get("mutable-aliasing").expect("mutable-aliasing missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::AliasViolation);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 3);
    assert_eq!(
        p.description(),
        "Cannot have multiple mutable references"
    );
    assert_eq!(
        p.c_pattern(),
        Some("T *a = ptr; T *b = ptr; *a = x; *b = y;")
    );
    assert_eq!(p.rust_error(), Some("E0499"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("single mutable reference"));
}

#[test]
fn level3_immut_mut_aliasing_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("immut-mut-aliasing")
        .expect("immut-mut-aliasing missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::AliasViolation);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 3);
    assert_eq!(
        p.description(),
        "Cannot have mutable reference while immutable exists"
    );
    assert_eq!(
        p.c_pattern(),
        Some("const T *r = ptr; *ptr = x; use(r);")
    );
    assert_eq!(p.rust_error(), Some("E0502"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("End immutable borrow"));
}

#[test]
fn level3_use_after_free_pattern() {
    let library = default_pattern_library();
    let p = library.get("use-after-free").expect("use-after-free missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::DanglingPointerRisk);
    assert_eq!(p.severity(), ErrorSeverity::Critical);
    assert_eq!(p.curriculum_level(), 3);
    assert_eq!(
        p.description(),
        "Use of pointer after free causes undefined behavior"
    );
    assert_eq!(p.c_pattern(), Some("free(ptr); use(ptr);"));
    assert_eq!(p.rust_error(), Some("E0382"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("Option<Box<T>>"));
    assert!(fix.code_template().contains("take()"));
}

#[test]
fn level3_return_local_ref_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("return-local-ref")
        .expect("return-local-ref missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::DanglingPointerRisk);
    assert_eq!(p.severity(), ErrorSeverity::Critical);
    assert_eq!(p.curriculum_level(), 3);
    assert_eq!(
        p.description(),
        "Returning pointer to local variable is undefined behavior"
    );
    assert_eq!(
        p.c_pattern(),
        Some("int* foo() { int x = 1; return &x; }")
    );
    assert_eq!(p.rust_error(), Some("E0515"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("Return owned value"));
}

// ============================================================================
// LEVEL 4: Resource Management
// ============================================================================

#[test]
fn level4_missing_free_pattern() {
    let library = default_pattern_library();
    let p = library.get("missing-free").expect("missing-free missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::ResourceLeakPattern);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 4);
    assert_eq!(
        p.description(),
        "Allocated memory not freed causes leak"
    );
    assert_eq!(
        p.c_pattern(),
        Some("void* p = malloc(...); return; // leak!")
    );
    assert!(p.rust_error().is_none());
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("RAII"));
    assert!(fix.code_template().contains("Box::new"));
}

#[test]
fn level4_file_handle_leak_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("file-handle-leak")
        .expect("file-handle-leak missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::ResourceLeakPattern);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 4);
    assert_eq!(
        p.description(),
        "File handle not closed causes resource leak"
    );
    assert_eq!(
        p.c_pattern(),
        Some("FILE *f = fopen(...); return; // leak!")
    );
    assert!(p.rust_error().is_none());
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("File type"));
    assert!(fix.code_template().contains("File::open"));
}

#[test]
fn level4_unnecessary_unsafe_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("unnecessary-unsafe")
        .expect("unnecessary-unsafe missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::UnsafeMinimizationFailure);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 4);
    assert_eq!(
        p.description(),
        "Safe alternative exists for this unsafe operation"
    );
    assert_eq!(
        p.c_pattern(),
        Some("*(ptr + i) = value; // pointer arithmetic")
    );
    assert!(p.rust_error().is_none());
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("safe slice indexing"));
    assert!(fix.code_template().contains("slice[i]"));
}

#[test]
fn level4_null_check_to_option_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("null-check-to-option")
        .expect("null-check-to-option missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::UnsafeMinimizationFailure);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 4);
    assert_eq!(
        p.description(),
        "Null pointer check should use Option<T>"
    );
    assert_eq!(
        p.c_pattern(),
        Some("if (ptr != NULL) { use(ptr); }")
    );
    assert!(p.rust_error().is_none());
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("Option<T>"));
    assert!(fix.code_template().contains("if let Some"));
}

// ============================================================================
// LEVEL 5: Complex Patterns (Expert)
// ============================================================================

#[test]
fn level5_self_referential_struct_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("self-referential-struct")
        .expect("self-referential-struct missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::AliasViolation);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 5);
    assert_eq!(
        p.description(),
        "Self-referential struct needs Pin or unsafe"
    );
    assert_eq!(
        p.c_pattern(),
        Some("struct Node { struct Node *next; int data; }")
    );
    assert_eq!(p.rust_error(), Some("E0597"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("Box"));
    assert!(fix.code_template().contains("Option<Box<Node>>"));
}

#[test]
fn level5_multiple_lifetimes_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("multiple-lifetimes")
        .expect("multiple-lifetimes missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::LifetimeInferenceGap);
    assert_eq!(p.severity(), ErrorSeverity::Error);
    assert_eq!(p.curriculum_level(), 5);
    assert_eq!(
        p.description(),
        "Function with multiple reference params needs explicit lifetimes"
    );
    assert_eq!(
        p.c_pattern(),
        Some("T* pick(T *a, T *b, int cond)")
    );
    assert_eq!(p.rust_error(), Some("E0106"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("lifetime bounds"));
    assert!(fix.code_template().contains("<'a>"));
}

#[test]
fn level5_interior_mutability_pattern() {
    let library = default_pattern_library();
    let p = library
        .get("interior-mutability")
        .expect("interior-mutability missing");
    assert_eq!(p.error_kind(), OwnershipErrorKind::MutabilityMismatch);
    assert_eq!(p.severity(), ErrorSeverity::Warning);
    assert_eq!(p.curriculum_level(), 5);
    assert_eq!(
        p.description(),
        "Mutation through shared reference needs Cell/RefCell"
    );
    assert_eq!(
        p.c_pattern(),
        Some("void inc(Counter *c) { c->count++; } // called via const ptr")
    );
    assert_eq!(p.rust_error(), Some("E0596"));
    let fix = p.suggested_fix().expect("should have fix");
    assert!(fix.description().contains("Cell<T>"));
    assert!(fix.code_template().contains("Cell<i32>"));
}

// ============================================================================
// ERROR KIND DISTRIBUTION: Every OwnershipErrorKind is represented
// ============================================================================

#[test]
fn default_library_covers_all_8_error_kinds() {
    let library = default_pattern_library();
    let error_kinds = [
        OwnershipErrorKind::PointerMisclassification,
        OwnershipErrorKind::LifetimeInferenceGap,
        OwnershipErrorKind::DanglingPointerRisk,
        OwnershipErrorKind::AliasViolation,
        OwnershipErrorKind::UnsafeMinimizationFailure,
        OwnershipErrorKind::ArraySliceMismatch,
        OwnershipErrorKind::ResourceLeakPattern,
        OwnershipErrorKind::MutabilityMismatch,
    ];
    for kind in &error_kinds {
        let patterns = library.get_by_error_kind(*kind);
        assert!(
            !patterns.is_empty(),
            "No patterns for error kind: {:?}",
            kind
        );
    }
}

#[test]
fn default_library_pointer_misclassification_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::PointerMisclassification);
    assert_eq!(patterns.len(), 2, "Should have 2 PointerMisclassification patterns");
}

#[test]
fn default_library_lifetime_inference_gap_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::LifetimeInferenceGap);
    assert_eq!(patterns.len(), 3, "Should have 3 LifetimeInferenceGap patterns");
}

#[test]
fn default_library_dangling_pointer_risk_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::DanglingPointerRisk);
    assert_eq!(patterns.len(), 2, "Should have 2 DanglingPointerRisk patterns");
}

#[test]
fn default_library_alias_violation_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::AliasViolation);
    assert_eq!(patterns.len(), 3, "Should have 3 AliasViolation patterns");
}

#[test]
fn default_library_unsafe_minimization_failure_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::UnsafeMinimizationFailure);
    assert_eq!(patterns.len(), 2, "Should have 2 UnsafeMinimizationFailure patterns");
}

#[test]
fn default_library_array_slice_mismatch_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::ArraySliceMismatch);
    assert_eq!(patterns.len(), 1, "Should have 1 ArraySliceMismatch pattern");
}

#[test]
fn default_library_resource_leak_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::ResourceLeakPattern);
    assert_eq!(patterns.len(), 2, "Should have 2 ResourceLeakPattern patterns");
}

#[test]
fn default_library_mutability_mismatch_count() {
    let library = default_pattern_library();
    let patterns = library.get_by_error_kind(OwnershipErrorKind::MutabilityMismatch);
    assert_eq!(patterns.len(), 2, "Should have 2 MutabilityMismatch patterns");
}

// ============================================================================
// CURRICULUM ORDERING: Patterns are ordered correctly
// ============================================================================

#[test]
fn default_library_curriculum_ordered_starts_at_level_1() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();
    assert!(!ordered.is_empty());
    assert_eq!(ordered[0].curriculum_level(), 1);
}

#[test]
fn default_library_curriculum_ordered_monotonically_non_decreasing() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();
    for window in ordered.windows(2) {
        assert!(
            window[0].curriculum_level() <= window[1].curriculum_level(),
            "Curriculum order violated: {} (level {}) before {} (level {})",
            window[0].id(),
            window[0].curriculum_level(),
            window[1].id(),
            window[1].curriculum_level()
        );
    }
}

#[test]
fn default_library_curriculum_levels_span_1_through_5() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();
    let levels: std::collections::HashSet<u8> =
        ordered.iter().map(|p| p.curriculum_level()).collect();
    for level in 1..=5 {
        assert!(
            levels.contains(&level),
            "Missing curriculum level {}",
            level
        );
    }
}

#[test]
fn default_library_level_distribution() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();
    let count_by_level = |level: u8| -> usize {
        ordered.iter().filter(|p| p.curriculum_level() == level).count()
    };
    assert_eq!(count_by_level(1), 3, "Level 1 should have 3 patterns");
    assert_eq!(count_by_level(2), 3, "Level 2 should have 3 patterns");
    assert_eq!(count_by_level(3), 4, "Level 3 should have 4 patterns");
    assert_eq!(count_by_level(4), 4, "Level 4 should have 4 patterns");
    assert_eq!(count_by_level(5), 3, "Level 5 should have 3 patterns");
}

// ============================================================================
// SEVERITY DISTRIBUTION
// ============================================================================

#[test]
fn default_library_critical_patterns() {
    let library = default_pattern_library();
    let critical: Vec<_> = library
        .iter()
        .filter(|p| p.severity() == ErrorSeverity::Critical)
        .collect();
    assert_eq!(critical.len(), 2, "Should have 2 Critical severity patterns");
    // Both are DanglingPointerRisk
    for p in &critical {
        assert_eq!(
            p.error_kind(),
            OwnershipErrorKind::DanglingPointerRisk,
            "Critical patterns should be DanglingPointerRisk: {}",
            p.id()
        );
    }
}

#[test]
fn default_library_error_severity_patterns() {
    let library = default_pattern_library();
    let errors: Vec<_> = library
        .iter()
        .filter(|p| p.severity() == ErrorSeverity::Error)
        .collect();
    assert_eq!(errors.len(), 6, "Should have 6 Error severity patterns");
}

#[test]
fn default_library_warning_severity_patterns() {
    let library = default_pattern_library();
    let warnings: Vec<_> = library
        .iter()
        .filter(|p| p.severity() == ErrorSeverity::Warning)
        .collect();
    assert_eq!(warnings.len(), 9, "Should have 9 Warning severity patterns");
}

// ============================================================================
// RUST ERROR MATCHING: Patterns with rust_error can be matched
// ============================================================================

#[test]
fn default_library_match_e0308_finds_patterns() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0308: mismatched types");
    assert!(
        !matches.is_empty(),
        "Should match patterns with E0308 rust_error"
    );
}

#[test]
fn default_library_match_e0106_finds_lifetime_patterns() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0106: missing lifetime specifier");
    assert!(
        !matches.is_empty(),
        "Should match E0106 patterns"
    );
    for p in &matches {
        assert_eq!(p.error_kind(), OwnershipErrorKind::LifetimeInferenceGap);
    }
}

#[test]
fn default_library_match_e0499_finds_alias_pattern() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0499: cannot borrow");
    assert!(!matches.is_empty(), "Should match E0499 pattern");
}

#[test]
fn default_library_match_e0502_finds_immut_mut_pattern() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0502: cannot borrow");
    assert!(!matches.is_empty(), "Should match E0502 pattern");
}

#[test]
fn default_library_match_e0382_finds_use_after_free() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0382: use of moved value");
    assert!(!matches.is_empty(), "Should match E0382 pattern");
}

#[test]
fn default_library_match_e0515_finds_return_local() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0515: cannot return reference");
    assert!(!matches.is_empty(), "Should match E0515 pattern");
}

#[test]
fn default_library_match_e0596_finds_mutability_patterns() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0596: cannot borrow");
    assert!(!matches.is_empty(), "Should match E0596 patterns");
}

#[test]
fn default_library_match_e0597_finds_self_referential() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E0597: borrowed value does not live long enough");
    assert!(!matches.is_empty(), "Should match E0597 pattern");
}

#[test]
fn default_library_match_unknown_error_returns_empty() {
    let library = default_pattern_library();
    let matches = library.match_rust_error("E9999: completely unknown");
    assert!(matches.is_empty(), "Unknown error should not match");
}

// ============================================================================
// ALL PATTERNS: Verify c_pattern, suggested_fix, and occurrence_count
// ============================================================================

#[test]
fn all_patterns_have_c_patterns() {
    let library = default_pattern_library();
    for p in library.iter() {
        assert!(
            p.c_pattern().is_some(),
            "Pattern '{}' should have c_pattern",
            p.id()
        );
        assert!(
            !p.c_pattern().unwrap().is_empty(),
            "Pattern '{}' c_pattern should not be empty",
            p.id()
        );
    }
}

#[test]
fn all_patterns_have_suggested_fixes() {
    let library = default_pattern_library();
    for p in library.iter() {
        assert!(
            p.suggested_fix().is_some(),
            "Pattern '{}' should have suggested_fix",
            p.id()
        );
        let fix = p.suggested_fix().unwrap();
        assert!(
            !fix.description().is_empty(),
            "Pattern '{}' fix description should not be empty",
            p.id()
        );
        assert!(
            !fix.code_template().is_empty(),
            "Pattern '{}' fix code_template should not be empty",
            p.id()
        );
    }
}

#[test]
fn all_patterns_start_with_zero_occurrences() {
    let library = default_pattern_library();
    for p in library.iter() {
        assert_eq!(
            p.occurrence_count(),
            0,
            "Pattern '{}' should start with 0 occurrences",
            p.id()
        );
    }
}

// ============================================================================
// UNIQUE IDS: No duplicate pattern IDs
// ============================================================================

#[test]
fn all_pattern_ids_are_unique() {
    let library = default_pattern_library();
    let ids: Vec<&str> = library.iter().map(|p| p.id()).collect();
    let unique_ids: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(
        ids.len(),
        unique_ids.len(),
        "All pattern IDs should be unique"
    );
}

// ============================================================================
// SPECIFIC PATTERN IDS: All 18 are present by name
// ============================================================================

#[test]
fn all_17_pattern_ids_present() {
    let library = default_pattern_library();
    let expected_ids = [
        "malloc-to-box",
        "array-to-vec",
        "const-to-immut-ref",
        "missing-lifetime",
        "struct-lifetime",
        "array-param-to-slice",
        "mutable-aliasing",
        "immut-mut-aliasing",
        "use-after-free",
        "return-local-ref",
        "missing-free",
        "file-handle-leak",
        "unnecessary-unsafe",
        "null-check-to-option",
        "self-referential-struct",
        "multiple-lifetimes",
        "interior-mutability",
    ];
    assert_eq!(expected_ids.len(), 17);
    for id in &expected_ids {
        assert!(
            library.get(id).is_some(),
            "Missing pattern with ID: {}",
            id
        );
    }
}

// ============================================================================
// SUGGESTED FIX CONFIDENCE: All use default (0.5)
// ============================================================================

#[test]
fn all_suggested_fixes_have_default_confidence() {
    let library = default_pattern_library();
    for p in library.iter() {
        if let Some(fix) = p.suggested_fix() {
            assert!(
                (fix.confidence() - 0.5).abs() < f32::EPSILON,
                "Pattern '{}' fix should have default 0.5 confidence, got {}",
                p.id(),
                fix.confidence()
            );
        }
    }
}

// ============================================================================
// ITERATE AND COLLECT: Force full construction
// ============================================================================

#[test]
fn iterate_all_patterns_exercise_all_accessors() {
    let library = default_pattern_library();
    let mut total_desc_len = 0usize;
    let mut total_c_pattern_len = 0usize;
    let mut total_fix_desc_len = 0usize;
    let mut total_fix_template_len = 0usize;

    for p in library.iter() {
        // Exercise all accessors to ensure every field is read
        let _ = p.id();
        let _ = p.error_kind();
        let _ = p.description();
        let _ = p.c_pattern();
        let _ = p.rust_error();
        let _ = p.severity();
        let _ = p.curriculum_level();
        let _ = p.occurrence_count();

        total_desc_len += p.description().len();
        if let Some(c) = p.c_pattern() {
            total_c_pattern_len += c.len();
        }
        if let Some(fix) = p.suggested_fix() {
            let _ = fix.description();
            let _ = fix.code_template();
            let _ = fix.confidence();
            total_fix_desc_len += fix.description().len();
            total_fix_template_len += fix.code_template().len();
        }
    }

    assert!(total_desc_len > 200, "Combined descriptions should be substantial");
    assert!(total_c_pattern_len > 200, "Combined c_patterns should be substantial");
    assert!(total_fix_desc_len > 100, "Combined fix descriptions should be substantial");
    assert!(total_fix_template_len > 100, "Combined fix templates should be substantial");
}

// ============================================================================
// MUTABILITY: Record occurrences via get_mut
// ============================================================================

#[test]
fn default_library_record_occurrences_via_get_mut() {
    let mut library = default_pattern_library();
    {
        let p = library.get_mut("malloc-to-box").expect("pattern should exist");
        p.record_occurrence();
        p.record_occurrence();
    }
    let p = library.get("malloc-to-box").unwrap();
    assert_eq!(p.occurrence_count(), 2);
}

// ============================================================================
// ERROR KIND TO DEFECT MAPPING: Integration with OwnershipDefect taxonomy
// ============================================================================

#[test]
fn default_library_error_kinds_map_to_defects() {
    let library = default_pattern_library();
    for p in library.iter() {
        let defect = p.error_kind().to_defect();
        // Verify the defect code is non-empty
        assert!(!defect.code().is_empty(), "Defect code empty for pattern {}", p.id());
        assert!(!defect.description().is_empty(), "Defect description empty for pattern {}", p.id());
    }
}
