//! Corpus diversity validation for training data quality
//!
//! Implements Genchi Genbutsu (Go and See) principle - validating that
//! the training corpus represents real-world C code diversity.
//!
//! # References
//! - training-oracle-spec.md §1.4: Corpus Diversity Validation
//! - Gemini Review: "Ensure error distribution matches broader C code"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Error histogram for a corpus
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorHistogram {
    /// Total files analyzed
    pub total_files: usize,
    /// Total lines of code
    pub total_loc: usize,
    /// Error counts by error code (e.g., "E0382" -> 15)
    pub by_error_code: HashMap<String, usize>,
    /// Error counts by category
    pub by_category: HashMap<ErrorCategory, usize>,
    /// C construct coverage
    pub construct_coverage: HashMap<CConstruct, usize>,
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Ownership errors (E0382, E0505, E0506)
    Ownership,
    /// Borrowing errors (E0499, E0502, E0503)
    Borrowing,
    /// Lifetime errors (E0597, E0515, E0716)
    Lifetime,
    /// Type errors (E0308, E0277)
    Type,
    /// Other errors
    Other,
}

/// C constructs for coverage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CConstruct {
    /// Raw pointers (*T, *const T, *mut T)
    RawPointer,
    /// Arrays (T[N])
    Array,
    /// Malloc/free patterns
    MallocFree,
    /// Function pointers
    FunctionPointer,
    /// Struct definitions
    Struct,
    /// Union definitions
    Union,
    /// Enum definitions
    Enum,
    /// Typedef aliases
    Typedef,
    /// Preprocessor macros
    Macro,
    /// Goto statements
    Goto,
    /// Switch statements
    Switch,
    /// For loops
    ForLoop,
    /// While loops
    WhileLoop,
    /// Do-while loops
    DoWhile,
}

impl ErrorHistogram {
    /// Create a new empty histogram
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error
    pub fn record_error(&mut self, error_code: &str) {
        *self.by_error_code.entry(error_code.to_string()).or_default() += 1;
        let category = categorize_error(error_code);
        *self.by_category.entry(category).or_default() += 1;
    }

    /// Record a C construct occurrence
    pub fn record_construct(&mut self, construct: CConstruct) {
        *self.construct_coverage.entry(construct).or_default() += 1;
    }

    /// Get total error count
    pub fn total_errors(&self) -> usize {
        self.by_error_code.values().sum()
    }

    /// Get error distribution as probabilities
    pub fn error_distribution(&self) -> HashMap<String, f64> {
        let total = self.total_errors() as f64;
        if total == 0.0 {
            return HashMap::new();
        }
        self.by_error_code
            .iter()
            .map(|(code, count)| (code.clone(), *count as f64 / total))
            .collect()
    }

    /// Get category distribution as probabilities
    pub fn category_distribution(&self) -> HashMap<ErrorCategory, f64> {
        let total: usize = self.by_category.values().sum();
        if total == 0 {
            return HashMap::new();
        }
        self.by_category
            .iter()
            .map(|(cat, count)| (*cat, *count as f64 / total as f64))
            .collect()
    }
}

/// Categorize an error code
pub fn categorize_error(error_code: &str) -> ErrorCategory {
    match error_code {
        "E0382" | "E0505" | "E0506" | "E0507" => ErrorCategory::Ownership,
        "E0499" | "E0502" | "E0503" | "E0500" => ErrorCategory::Borrowing,
        "E0597" | "E0515" | "E0716" | "E0623" | "E0106" => ErrorCategory::Lifetime,
        "E0308" | "E0277" | "E0369" => ErrorCategory::Type,
        _ => ErrorCategory::Other,
    }
}

/// Diversity metrics for comparing corpora
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityMetrics {
    /// Jensen-Shannon divergence between distributions
    pub js_divergence: f64,
    /// Kullback-Leibler divergence (asymmetric)
    pub kl_divergence: f64,
    /// Overlap coefficient (Szymkiewicz-Simpson)
    pub overlap: f64,
    /// Number of unique error codes in primary corpus
    pub primary_unique_errors: usize,
    /// Number of unique error codes in comparison corpus
    pub comparison_unique_errors: usize,
    /// Number of shared error codes
    pub shared_error_codes: usize,
    /// Coverage ratio: shared / union
    pub coverage_ratio: f64,
    /// Passes threshold check
    pub passes_threshold: bool,
}

impl DiversityMetrics {
    /// Check if diversity is acceptable
    /// Per spec: JS divergence < 0.15
    pub fn is_acceptable(&self) -> bool {
        self.js_divergence < 0.15 && self.coverage_ratio > 0.6
    }
}

/// Compare two error histograms for diversity
pub fn compare_histograms(primary: &ErrorHistogram, comparison: &ErrorHistogram) -> DiversityMetrics {
    let p_dist = primary.error_distribution();
    let q_dist = comparison.error_distribution();

    // Get all unique error codes
    let all_codes: std::collections::HashSet<_> = p_dist
        .keys()
        .chain(q_dist.keys())
        .cloned()
        .collect();

    let shared: usize = p_dist
        .keys()
        .filter(|k| q_dist.contains_key(*k))
        .count();

    // Calculate JS divergence
    let js_divergence = jensen_shannon_divergence(&p_dist, &q_dist, &all_codes);
    let kl_divergence = kl_divergence_safe(&p_dist, &q_dist, &all_codes);

    // Calculate overlap coefficient
    let overlap = if p_dist.len().min(q_dist.len()) > 0 {
        shared as f64 / p_dist.len().min(q_dist.len()) as f64
    } else {
        0.0
    };

    let coverage_ratio = if all_codes.is_empty() {
        1.0 // Empty distributions are considered fully covered
    } else {
        shared as f64 / all_codes.len() as f64
    };

    let metrics = DiversityMetrics {
        js_divergence,
        kl_divergence,
        overlap,
        primary_unique_errors: p_dist.len(),
        comparison_unique_errors: q_dist.len(),
        shared_error_codes: shared,
        coverage_ratio,
        passes_threshold: false, // Will be set below
    };

    DiversityMetrics {
        passes_threshold: metrics.is_acceptable(),
        ..metrics
    }
}

/// Calculate Jensen-Shannon divergence between two distributions
fn jensen_shannon_divergence(
    p: &HashMap<String, f64>,
    q: &HashMap<String, f64>,
    all_keys: &std::collections::HashSet<String>,
) -> f64 {
    if all_keys.is_empty() {
        return 0.0;
    }

    // Create mixture distribution M = (P + Q) / 2
    let mut m = HashMap::new();
    for key in all_keys {
        let p_val = p.get(key).copied().unwrap_or(0.0);
        let q_val = q.get(key).copied().unwrap_or(0.0);
        m.insert(key.clone(), (p_val + q_val) / 2.0);
    }

    // JS(P || Q) = (KL(P || M) + KL(Q || M)) / 2
    let kl_pm = kl_divergence_safe(p, &m, all_keys);
    let kl_qm = kl_divergence_safe(q, &m, all_keys);

    (kl_pm + kl_qm) / 2.0
}

/// Calculate KL divergence with smoothing for zero probabilities
fn kl_divergence_safe(
    p: &HashMap<String, f64>,
    q: &HashMap<String, f64>,
    all_keys: &std::collections::HashSet<String>,
) -> f64 {
    if all_keys.is_empty() {
        return 0.0;
    }

    let epsilon = 1e-10; // Smoothing factor
    let mut kl = 0.0;

    for key in all_keys {
        let p_val = p.get(key).copied().unwrap_or(0.0);
        let q_val = q.get(key).copied().unwrap_or(0.0);

        // Skip if p is zero (0 * log(0/q) = 0)
        if p_val > epsilon {
            // Add smoothing to q to avoid log(0)
            let q_smooth = q_val.max(epsilon);
            kl += p_val * (p_val / q_smooth).ln();
        }
    }

    kl
}

/// Configuration for diversity validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityConfig {
    /// Maximum JS divergence allowed
    pub max_js_divergence: f64,
    /// Minimum coverage ratio required
    pub min_coverage_ratio: f64,
    /// Minimum number of error codes required
    pub min_error_codes: usize,
}

impl Default for DiversityConfig {
    fn default() -> Self {
        Self {
            max_js_divergence: 0.15,
            min_coverage_ratio: 0.6,
            min_error_codes: 5,
        }
    }
}

/// Validation result for a corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityValidation {
    /// Primary corpus histogram
    pub primary_histogram: ErrorHistogram,
    /// Comparison histograms
    pub comparisons: Vec<(String, ErrorHistogram, DiversityMetrics)>,
    /// Overall validation status
    pub passed: bool,
    /// Issues found
    pub issues: Vec<String>,
}

impl DiversityValidation {
    /// Create a new validation
    pub fn new(primary: ErrorHistogram) -> Self {
        Self {
            primary_histogram: primary,
            comparisons: Vec::new(),
            passed: true,
            issues: Vec::new(),
        }
    }

    /// Add a comparison corpus
    pub fn add_comparison(&mut self, name: &str, histogram: ErrorHistogram, config: &DiversityConfig) {
        let metrics = compare_histograms(&self.primary_histogram, &histogram);

        // Check thresholds
        if metrics.js_divergence > config.max_js_divergence {
            self.issues.push(format!(
                "{}: JS divergence {:.3} exceeds threshold {:.3}",
                name, metrics.js_divergence, config.max_js_divergence
            ));
            self.passed = false;
        }

        if metrics.coverage_ratio < config.min_coverage_ratio {
            self.issues.push(format!(
                "{}: Coverage ratio {:.2} below threshold {:.2}",
                name, metrics.coverage_ratio, config.min_coverage_ratio
            ));
            self.passed = false;
        }

        self.comparisons.push((name.to_string(), histogram, metrics));
    }

    /// Generate a validation report
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        report.push_str("## Corpus Diversity Validation Report\n\n");

        report.push_str(&format!(
            "**Primary Corpus**: {} files, {} LOC, {} errors\n\n",
            self.primary_histogram.total_files,
            self.primary_histogram.total_loc,
            self.primary_histogram.total_errors()
        ));

        if self.comparisons.is_empty() {
            report.push_str("*No comparison corpora provided*\n\n");
        } else {
            report.push_str("### Comparison Results\n\n");
            report.push_str("| Corpus | JS Divergence | Coverage | Status |\n");
            report.push_str("|--------|---------------|----------|--------|\n");

            for (name, _, metrics) in &self.comparisons {
                let status = if metrics.passes_threshold {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                };
                report.push_str(&format!(
                    "| {} | {:.3} | {:.1}% | {} |\n",
                    name,
                    metrics.js_divergence,
                    metrics.coverage_ratio * 100.0,
                    status
                ));
            }
        }

        report.push_str("\n### Overall Status: ");
        if self.passed {
            report.push_str("✅ PASSED\n");
        } else {
            report.push_str("❌ FAILED\n\n");
            report.push_str("**Issues:**\n");
            for issue in &self.issues {
                report.push_str(&format!("- {}\n", issue));
            }
        }

        report
    }
}

/// Analyze a directory for error histogram (stub implementation)
pub fn analyze_corpus(path: &Path) -> Result<ErrorHistogram, std::io::Error> {
    use std::fs;

    let mut histogram = ErrorHistogram::new();

    // Walk directory and count C files
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("c"))
    {
        histogram.total_files += 1;

        // Count lines of code
        if let Ok(content) = fs::read_to_string(entry.path()) {
            histogram.total_loc += content.lines().count();

            // Detect C constructs (simple heuristic)
            if content.contains("malloc(") || content.contains("calloc(") {
                histogram.record_construct(CConstruct::MallocFree);
            }
            if content.contains("*") && (content.contains("int *") || content.contains("char *")) {
                histogram.record_construct(CConstruct::RawPointer);
            }
            if content.contains("struct ") {
                histogram.record_construct(CConstruct::Struct);
            }
            if content.contains("typedef ") {
                histogram.record_construct(CConstruct::Typedef);
            }
            if content.contains("switch (") || content.contains("switch(") {
                histogram.record_construct(CConstruct::Switch);
            }
            if content.contains("for (") || content.contains("for(") {
                histogram.record_construct(CConstruct::ForLoop);
            }
            if content.contains("while (") || content.contains("while(") {
                histogram.record_construct(CConstruct::WhileLoop);
            }
            if content.contains("goto ") {
                histogram.record_construct(CConstruct::Goto);
            }
        }
    }

    Ok(histogram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_histogram_new() {
        let hist = ErrorHistogram::new();
        assert_eq!(hist.total_files, 0);
        assert_eq!(hist.total_errors(), 0);
    }

    #[test]
    fn test_record_error() {
        let mut hist = ErrorHistogram::new();
        hist.record_error("E0382");
        hist.record_error("E0382");
        hist.record_error("E0499");

        assert_eq!(hist.total_errors(), 3);
        assert_eq!(hist.by_error_code.get("E0382"), Some(&2));
        assert_eq!(hist.by_error_code.get("E0499"), Some(&1));
    }

    #[test]
    fn test_categorize_ownership_errors() {
        assert_eq!(categorize_error("E0382"), ErrorCategory::Ownership);
        assert_eq!(categorize_error("E0505"), ErrorCategory::Ownership);
        assert_eq!(categorize_error("E0506"), ErrorCategory::Ownership);
    }

    #[test]
    fn test_categorize_borrowing_errors() {
        assert_eq!(categorize_error("E0499"), ErrorCategory::Borrowing);
        assert_eq!(categorize_error("E0502"), ErrorCategory::Borrowing);
        assert_eq!(categorize_error("E0503"), ErrorCategory::Borrowing);
    }

    #[test]
    fn test_categorize_lifetime_errors() {
        assert_eq!(categorize_error("E0597"), ErrorCategory::Lifetime);
        assert_eq!(categorize_error("E0515"), ErrorCategory::Lifetime);
        assert_eq!(categorize_error("E0716"), ErrorCategory::Lifetime);
    }

    #[test]
    fn test_categorize_type_errors() {
        assert_eq!(categorize_error("E0308"), ErrorCategory::Type);
        assert_eq!(categorize_error("E0277"), ErrorCategory::Type);
    }

    #[test]
    fn test_categorize_other_errors() {
        assert_eq!(categorize_error("E9999"), ErrorCategory::Other);
        assert_eq!(categorize_error("unknown"), ErrorCategory::Other);
    }

    #[test]
    fn test_error_distribution() {
        let mut hist = ErrorHistogram::new();
        hist.record_error("E0382");
        hist.record_error("E0382");
        hist.record_error("E0499");
        hist.record_error("E0499");

        let dist = hist.error_distribution();
        assert!((dist.get("E0382").unwrap() - 0.5).abs() < 0.001);
        assert!((dist.get("E0499").unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_category_distribution() {
        let mut hist = ErrorHistogram::new();
        hist.record_error("E0382"); // Ownership
        hist.record_error("E0499"); // Borrowing

        let dist = hist.category_distribution();
        assert!((dist.get(&ErrorCategory::Ownership).unwrap() - 0.5).abs() < 0.001);
        assert!((dist.get(&ErrorCategory::Borrowing).unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_jensen_shannon_identical() {
        let mut p = HashMap::new();
        p.insert("E0382".to_string(), 0.5);
        p.insert("E0499".to_string(), 0.5);

        let keys: std::collections::HashSet<_> = p.keys().cloned().collect();
        let js = jensen_shannon_divergence(&p, &p, &keys);

        // Identical distributions should have JS divergence of 0
        assert!(js < 0.001);
    }

    #[test]
    fn test_jensen_shannon_different() {
        let mut p = HashMap::new();
        p.insert("E0382".to_string(), 1.0);

        let mut q = HashMap::new();
        q.insert("E0499".to_string(), 1.0);

        let keys: std::collections::HashSet<_> = p.keys().chain(q.keys()).cloned().collect();
        let js = jensen_shannon_divergence(&p, &q, &keys);

        // Completely different distributions should have JS divergence close to ln(2)
        assert!(js > 0.5);
    }

    #[test]
    fn test_compare_histograms_identical() {
        let mut hist = ErrorHistogram::new();
        hist.record_error("E0382");
        hist.record_error("E0499");

        let metrics = compare_histograms(&hist, &hist);

        assert!(metrics.js_divergence < 0.001);
        assert!((metrics.coverage_ratio - 1.0).abs() < 0.001);
        assert!(metrics.is_acceptable());
    }

    #[test]
    fn test_compare_histograms_partial_overlap() {
        let mut hist1 = ErrorHistogram::new();
        hist1.record_error("E0382");
        hist1.record_error("E0499");
        hist1.record_error("E0597");

        let mut hist2 = ErrorHistogram::new();
        hist2.record_error("E0382");
        hist2.record_error("E0499");
        hist2.record_error("E0308");

        let metrics = compare_histograms(&hist1, &hist2);

        // Should have 2 shared codes out of 4 total
        assert_eq!(metrics.shared_error_codes, 2);
        assert!(metrics.js_divergence < 0.5); // Some divergence but not extreme
    }

    #[test]
    fn test_diversity_config_default() {
        let config = DiversityConfig::default();
        assert!((config.max_js_divergence - 0.15).abs() < 0.001);
        assert!((config.min_coverage_ratio - 0.6).abs() < 0.001);
        assert_eq!(config.min_error_codes, 5);
    }

    #[test]
    fn test_diversity_metrics_acceptable() {
        let metrics = DiversityMetrics {
            js_divergence: 0.1,
            kl_divergence: 0.2,
            overlap: 0.8,
            primary_unique_errors: 10,
            comparison_unique_errors: 10,
            shared_error_codes: 8,
            coverage_ratio: 0.8,
            passes_threshold: true,
        };

        assert!(metrics.is_acceptable());
    }

    #[test]
    fn test_diversity_metrics_unacceptable_divergence() {
        let metrics = DiversityMetrics {
            js_divergence: 0.25, // Too high
            kl_divergence: 0.3,
            overlap: 0.8,
            primary_unique_errors: 10,
            comparison_unique_errors: 10,
            shared_error_codes: 8,
            coverage_ratio: 0.8,
            passes_threshold: false,
        };

        assert!(!metrics.is_acceptable());
    }

    #[test]
    fn test_diversity_metrics_unacceptable_coverage() {
        let metrics = DiversityMetrics {
            js_divergence: 0.1,
            kl_divergence: 0.2,
            overlap: 0.3,
            primary_unique_errors: 10,
            comparison_unique_errors: 10,
            shared_error_codes: 3,
            coverage_ratio: 0.3, // Too low
            passes_threshold: false,
        };

        assert!(!metrics.is_acceptable());
    }

    #[test]
    fn test_diversity_validation_new() {
        let hist = ErrorHistogram::new();
        let validation = DiversityValidation::new(hist);

        assert!(validation.passed);
        assert!(validation.issues.is_empty());
        assert!(validation.comparisons.is_empty());
    }

    #[test]
    fn test_diversity_validation_add_comparison_pass() {
        let mut primary = ErrorHistogram::new();
        primary.record_error("E0382");
        primary.record_error("E0499");

        let mut comparison = ErrorHistogram::new();
        comparison.record_error("E0382");
        comparison.record_error("E0499");

        let mut validation = DiversityValidation::new(primary);
        validation.add_comparison("test", comparison, &DiversityConfig::default());

        assert!(validation.passed);
        assert!(validation.issues.is_empty());
    }

    #[test]
    fn test_diversity_validation_add_comparison_fail() {
        let mut primary = ErrorHistogram::new();
        primary.record_error("E0382");

        let mut comparison = ErrorHistogram::new();
        comparison.record_error("E9999");

        let mut validation = DiversityValidation::new(primary);
        validation.add_comparison("test", comparison, &DiversityConfig::default());

        assert!(!validation.passed);
        assert!(!validation.issues.is_empty());
    }

    #[test]
    fn test_diversity_validation_report() {
        let mut hist = ErrorHistogram::new();
        hist.total_files = 10;
        hist.total_loc = 1000;
        hist.record_error("E0382");

        let validation = DiversityValidation::new(hist);
        let report = validation.to_report();

        assert!(report.contains("Corpus Diversity Validation Report"));
        assert!(report.contains("10 files"));
        assert!(report.contains("1000 LOC"));
    }

    #[test]
    fn test_record_construct() {
        let mut hist = ErrorHistogram::new();
        hist.record_construct(CConstruct::RawPointer);
        hist.record_construct(CConstruct::RawPointer);
        hist.record_construct(CConstruct::Struct);

        assert_eq!(hist.construct_coverage.get(&CConstruct::RawPointer), Some(&2));
        assert_eq!(hist.construct_coverage.get(&CConstruct::Struct), Some(&1));
    }

    #[test]
    fn test_empty_distribution() {
        let hist = ErrorHistogram::new();
        let dist = hist.error_distribution();
        assert!(dist.is_empty());
    }

    #[test]
    fn test_empty_category_distribution() {
        let hist = ErrorHistogram::new();
        let dist = hist.category_distribution();
        assert!(dist.is_empty());
    }

    #[test]
    fn test_compare_empty_histograms() {
        let hist1 = ErrorHistogram::new();
        let hist2 = ErrorHistogram::new();

        let metrics = compare_histograms(&hist1, &hist2);

        assert!(metrics.js_divergence < 0.001);
        assert!((metrics.coverage_ratio - 1.0).abs() < 0.001); // Empty = fully covered
    }
}
