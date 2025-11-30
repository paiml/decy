//! CITL (Compiler-in-the-Loop) Pattern Mining for C-to-Rust Transpilation
//!
//! Uses entrenar's CITL module for Tarantula fault localization to identify
//! which C language features most strongly correlate with transpilation failures.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────┐     ┌──────────────────┐
//! │  C Source Files     │────►│  CorpusCITL      │
//! │  (transpilation)    │     │  Pattern Mining  │
//! └─────────────────────┘     └──────────────────┘
//!                                      │
//!                                      ▼
//!                              ┌──────────────────┐
//!                              │  DecisionCITL    │
//!                              │  (Tarantula)     │
//!                              └──────────────────┘
//!                                      │
//!                                      ▼
//!                              ┌──────────────────┐
//!                              │  Suspiciousness  │
//!                              │  Scores          │
//!                              └──────────────────┘
//! ```
//!
//! # References
//!
//! - Jones & Harrold (2005): Tarantula Fault Localization
//! - Zeller (2002): Isolating Cause-Effect Chains

use crate::error::OracleError;
use entrenar::citl::{CompilationOutcome, DecisionCITL, DecisionTrace};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Statistics for corpus ingestion
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestionStats {
    /// Total pairs processed
    pub total_pairs: usize,
    /// Pairs with successful Rust compilation
    pub success_pairs: usize,
    /// Pairs that failed compilation
    pub failed_pairs: usize,
    /// Unique C features discovered
    pub unique_features: usize,
}

/// C language feature detected during transpilation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CFeature {
    /// malloc/free pattern
    MallocFree,
    /// Pointer arithmetic (p + n, p++)
    PointerArithmetic,
    /// const char* string parameter
    ConstCharPointer,
    /// void* generic pointer
    VoidPointer,
    /// String iteration with while(*s)
    StringIteration,
    /// Array parameter with length
    ArrayWithLength,
    /// Struct with pointer fields
    StructWithPointers,
    /// Function pointer
    FunctionPointer,
    /// Global variable
    GlobalVariable,
    /// Static variable
    StaticVariable,
    /// Custom feature
    Custom(String),
}

impl CFeature {
    /// Convert to string for CITL decision tracking
    pub fn as_decision(&self) -> String {
        match self {
            CFeature::MallocFree => "malloc_free".to_string(),
            CFeature::PointerArithmetic => "pointer_arithmetic".to_string(),
            CFeature::ConstCharPointer => "const_char_pointer".to_string(),
            CFeature::VoidPointer => "void_pointer".to_string(),
            CFeature::StringIteration => "string_iteration".to_string(),
            CFeature::ArrayWithLength => "array_with_length".to_string(),
            CFeature::StructWithPointers => "struct_with_pointers".to_string(),
            CFeature::FunctionPointer => "function_pointer".to_string(),
            CFeature::GlobalVariable => "global_variable".to_string(),
            CFeature::StaticVariable => "static_variable".to_string(),
            CFeature::Custom(s) => s.clone(),
        }
    }

    /// Parse from decision string
    pub fn from_decision(s: &str) -> Self {
        match s {
            "malloc_free" => CFeature::MallocFree,
            "pointer_arithmetic" => CFeature::PointerArithmetic,
            "const_char_pointer" => CFeature::ConstCharPointer,
            "void_pointer" => CFeature::VoidPointer,
            "string_iteration" => CFeature::StringIteration,
            "array_with_length" => CFeature::ArrayWithLength,
            "struct_with_pointers" => CFeature::StructWithPointers,
            "function_pointer" => CFeature::FunctionPointer,
            "global_variable" => CFeature::GlobalVariable,
            "static_variable" => CFeature::StaticVariable,
            other => CFeature::Custom(other.to_string()),
        }
    }
}

/// Corpus-based CITL trainer for C-to-Rust pattern mining.
///
/// Ingests transpilation results and builds a pattern library
/// for fix suggestions using Tarantula fault localization.
pub struct CorpusCITL {
    citl: DecisionCITL,
    stats: IngestionStats,
    seen_features: HashSet<CFeature>,
}

impl CorpusCITL {
    /// Create a new CorpusCITL trainer.
    ///
    /// # Errors
    ///
    /// Returns error if entrenar CITL initialization fails.
    pub fn new() -> Result<Self, OracleError> {
        let citl = DecisionCITL::new()
            .map_err(|e| OracleError::PatternStoreError(e.to_string()))?;
        Ok(Self {
            citl,
            stats: IngestionStats::default(),
            seen_features: HashSet::new(),
        })
    }

    /// Ingest a C-to-Rust transpilation pair.
    ///
    /// # Arguments
    ///
    /// * `c_code` - Original C source code
    /// * `rust_code` - Transpiled Rust code (None if transpilation failed)
    /// * `features` - C features detected in the source
    ///
    /// # Errors
    ///
    /// Returns error if ingestion fails.
    pub fn ingest_pair(
        &mut self,
        _c_code: &str,
        rust_code: Option<&str>,
        features: &[CFeature],
    ) -> Result<(), OracleError> {
        // Track stats
        self.stats.total_pairs += 1;
        let success = rust_code.is_some();
        if success {
            self.stats.success_pairs += 1;
        } else {
            self.stats.failed_pairs += 1;
        }

        // Track unique features
        for feature in features {
            self.seen_features.insert(feature.clone());
        }
        self.stats.unique_features = self.seen_features.len();

        // Convert features to DecisionTraces
        let traces: Vec<DecisionTrace> = features
            .iter()
            .enumerate()
            .map(|(i, f)| {
                DecisionTrace::new(
                    format!("feature_{i}"),
                    f.as_decision(),
                    format!("C feature: {}", f.as_decision()),
                )
            })
            .collect();

        // Create outcome
        let outcome = if success {
            CompilationOutcome::success()
        } else {
            CompilationOutcome::failure(
                vec!["transpilation_failed".to_string()],
                vec![],
                vec!["Rust compilation failed".to_string()],
            )
        };

        // Ingest into CITL
        self.citl
            .ingest_session(traces, outcome, None)
            .map_err(|e| OracleError::PatternStoreError(e.to_string()))?;

        Ok(())
    }

    /// Get top suspicious C features by Tarantula score.
    ///
    /// # Arguments
    ///
    /// * `k` - Number of top features to return
    ///
    /// # Returns
    ///
    /// Vector of (feature, suspiciousness_score) pairs sorted by score descending.
    pub fn top_suspicious(&self, k: usize) -> Vec<(CFeature, f64)> {
        self.citl
            .top_suspicious_types(k)
            .into_iter()
            .map(|(decision, score)| (CFeature::from_decision(decision), f64::from(score)))
            .collect()
    }

    /// Get ingestion statistics.
    pub fn stats(&self) -> &IngestionStats {
        &self.stats
    }

    /// Extract C features from source code using pattern matching.
    ///
    /// This is a lightweight feature extractor that doesn't require
    /// full parsing - useful for quick corpus analysis.
    pub fn extract_features(&self, c_code: &str) -> Vec<CFeature> {
        let mut features = Vec::new();

        // malloc/free detection
        if c_code.contains("malloc") || c_code.contains("free(") {
            features.push(CFeature::MallocFree);
        }

        // Pointer arithmetic
        if c_code.contains("++") && c_code.contains("*")
            || c_code.contains("+ 1")
            || c_code.contains("+1")
        {
            // Check for pointer context
            if c_code.contains("char*")
                || c_code.contains("int*")
                || c_code.contains("void*")
                || c_code.contains("*p")
            {
                features.push(CFeature::PointerArithmetic);
            }
        }

        // const char* detection
        if c_code.contains("const char*") || c_code.contains("const char *") {
            features.push(CFeature::ConstCharPointer);
        }

        // void* detection
        if c_code.contains("void*") || c_code.contains("void *") {
            features.push(CFeature::VoidPointer);
        }

        // String iteration while(*s) or while(*p)
        if c_code.contains("while(*") || c_code.contains("while (*") {
            features.push(CFeature::StringIteration);
        }

        // Array with length parameter pattern: func(arr, len) or func(arr, n)
        if (c_code.contains("int n") || c_code.contains("size_t"))
            && (c_code.contains("[]") || c_code.contains("* arr"))
        {
            features.push(CFeature::ArrayWithLength);
        }

        // Struct with pointer fields
        if c_code.contains("struct") && c_code.contains("*") && c_code.contains("{") {
            features.push(CFeature::StructWithPointers);
        }

        // Function pointer
        if c_code.contains("(*") && c_code.contains(")(") {
            features.push(CFeature::FunctionPointer);
        }

        // Global variable (outside function, not in struct)
        // Simplified: look for assignment at file scope indicators
        if c_code.starts_with("int ") || c_code.starts_with("char ") {
            if !c_code.contains("(") {
                features.push(CFeature::GlobalVariable);
            }
        }

        // Static variable
        if c_code.contains("static ") {
            features.push(CFeature::StaticVariable);
        }

        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_citl_creation() {
        let citl = CorpusCITL::new();
        assert!(citl.is_ok(), "CorpusCITL creation should succeed");
    }

    #[test]
    fn test_ingest_successful_pair() {
        let mut citl = CorpusCITL::new().unwrap();

        let c_code = "int add(int a, int b) { return a + b; }";
        let rust_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let features = vec![];

        citl.ingest_pair(c_code, Some(rust_code), &features)
            .unwrap();

        assert_eq!(citl.stats().total_pairs, 1);
        assert_eq!(citl.stats().success_pairs, 1);
    }

    #[test]
    fn test_ingest_failed_pair() {
        let mut citl = CorpusCITL::new().unwrap();

        let c_code = "void* ptr = malloc(10); ptr++;";
        let features = vec![CFeature::MallocFree, CFeature::PointerArithmetic];

        citl.ingest_pair(c_code, None, &features).unwrap();

        assert_eq!(citl.stats().total_pairs, 1);
        assert_eq!(citl.stats().failed_pairs, 1);
    }

    #[test]
    fn test_top_suspicious_features() {
        let mut citl = CorpusCITL::new().unwrap();

        // Ingest failing pairs with pointer arithmetic
        for _ in 0..10 {
            citl.ingest_pair(
                "void* p = malloc(10); p++;",
                None, // Failed
                &[CFeature::MallocFree, CFeature::PointerArithmetic],
            )
            .unwrap();
        }

        // Ingest successful pairs without pointer arithmetic
        for _ in 0..10 {
            citl.ingest_pair(
                "int add(int a, int b) { return a + b; }",
                Some("fn add(a: i32, b: i32) -> i32 { a + b }"),
                &[],
            )
            .unwrap();
        }

        let suspicious = citl.top_suspicious(5);

        // Both MallocFree and PointerArithmetic have 100% failure rate
        assert!(!suspicious.is_empty());
        assert!(
            suspicious[0].1 > 0.5,
            "Top feature should have high suspiciousness"
        );
    }

    #[test]
    fn test_extract_features_malloc() {
        let citl = CorpusCITL::new().unwrap();

        let c_code = "int* p = malloc(sizeof(int)); free(p);";
        let features = citl.extract_features(c_code);

        assert!(features.contains(&CFeature::MallocFree));
    }

    #[test]
    fn test_extract_features_pointer_arithmetic() {
        let citl = CorpusCITL::new().unwrap();

        let c_code = "char* p = str; while(*p) { p++; }";
        let features = citl.extract_features(c_code);

        assert!(features.contains(&CFeature::PointerArithmetic));
        assert!(features.contains(&CFeature::StringIteration));
    }

    #[test]
    fn test_extract_features_const_char() {
        let citl = CorpusCITL::new().unwrap();

        let c_code = "void print(const char* msg) { printf(\"%s\", msg); }";
        let features = citl.extract_features(c_code);

        assert!(features.contains(&CFeature::ConstCharPointer));
    }

    #[test]
    fn test_cfeature_as_decision() {
        assert_eq!(CFeature::MallocFree.as_decision(), "malloc_free");
        assert_eq!(
            CFeature::PointerArithmetic.as_decision(),
            "pointer_arithmetic"
        );
        assert_eq!(
            CFeature::ConstCharPointer.as_decision(),
            "const_char_pointer"
        );
    }

    #[test]
    fn test_cfeature_roundtrip() {
        let features = vec![
            CFeature::MallocFree,
            CFeature::PointerArithmetic,
            CFeature::VoidPointer,
            CFeature::StringIteration,
        ];

        for f in features {
            let decision = f.as_decision();
            let recovered = CFeature::from_decision(&decision);
            assert_eq!(
                f, recovered,
                "Feature should roundtrip through decision string"
            );
        }
    }

    #[test]
    fn test_ingestion_stats_default() {
        let stats = IngestionStats::default();
        assert_eq!(stats.total_pairs, 0);
        assert_eq!(stats.success_pairs, 0);
        assert_eq!(stats.failed_pairs, 0);
    }

    #[test]
    fn test_unique_features_tracking() {
        let mut citl = CorpusCITL::new().unwrap();

        // Ingest with overlapping features
        citl.ingest_pair(
            "code1",
            None,
            &[CFeature::MallocFree, CFeature::VoidPointer],
        )
        .unwrap();
        citl.ingest_pair(
            "code2",
            None,
            &[CFeature::MallocFree, CFeature::PointerArithmetic],
        )
        .unwrap();

        // Should track 3 unique features: MallocFree, VoidPointer, PointerArithmetic
        assert_eq!(citl.stats().unique_features, 3);
    }
}
