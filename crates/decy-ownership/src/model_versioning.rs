//! Model versioning and rollback for ML-enhanced ownership inference (DECY-ML-017).
//!
//! Provides version management for ML models with rollback capability
//! following Toyota Way's Jidoka principle (stop the line on quality issues).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   MODEL VERSION MANAGER                         │
//! │                                                                 │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
//! │  │  v1.0.0     │  │  v1.1.0     │  │  v1.2.0     │  ← current  │
//! │  │  (baseline) │  │  (improved) │  │  (latest)   │             │
//! │  └─────────────┘  └─────────────┘  └─────────────┘             │
//! │        ▲                ▲                ▲                      │
//! │        │                │                │                      │
//! │        └────────────────┴────────────────┘                      │
//! │                         │                                       │
//! │                    ROLLBACK                                     │
//! │              (if quality degrades)                              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Jidoka (Stop the Line)
//!
//! If any quality metric degrades below threshold:
//! 1. Automatic rollback to previous version
//! 2. Alert generated for investigation
//! 3. Root cause documented before resuming

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Semantic version for models.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (new features)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl ModelVersion {
    /// Create a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Increment major version (resets minor and patch).
    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Increment minor version (resets patch).
    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Increment patch version.
    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    /// Parse from string (e.g., "1.2.3").
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim_start_matches('v').split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(Self::new(major, minor, patch))
    }
}

impl std::fmt::Display for ModelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for ModelVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// Quality metrics for a model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQualityMetrics {
    /// Classification accuracy (0.0 - 1.0)
    pub accuracy: f64,
    /// Precision (0.0 - 1.0)
    pub precision: f64,
    /// Recall (0.0 - 1.0)
    pub recall: f64,
    /// F1 score (0.0 - 1.0)
    pub f1_score: f64,
    /// Average confidence score
    pub avg_confidence: f64,
    /// Fallback rate (0.0 - 1.0)
    pub fallback_rate: f64,
    /// Number of validation samples
    pub sample_count: u64,
}

impl ModelQualityMetrics {
    /// Create new metrics.
    pub fn new(
        accuracy: f64,
        precision: f64,
        recall: f64,
        f1_score: f64,
        avg_confidence: f64,
        fallback_rate: f64,
        sample_count: u64,
    ) -> Self {
        Self {
            accuracy: accuracy.clamp(0.0, 1.0),
            precision: precision.clamp(0.0, 1.0),
            recall: recall.clamp(0.0, 1.0),
            f1_score: f1_score.clamp(0.0, 1.0),
            avg_confidence: avg_confidence.clamp(0.0, 1.0),
            fallback_rate: fallback_rate.clamp(0.0, 1.0),
            sample_count,
        }
    }

    /// Check if metrics meet minimum thresholds.
    pub fn meets_thresholds(&self, thresholds: &QualityThresholds) -> bool {
        self.accuracy >= thresholds.min_accuracy
            && self.precision >= thresholds.min_precision
            && self.recall >= thresholds.min_recall
            && self.f1_score >= thresholds.min_f1
    }

    /// Check if this version is better than another.
    pub fn is_better_than(&self, other: &Self) -> bool {
        // Primary: accuracy, secondary: F1
        if (self.accuracy - other.accuracy).abs() > 0.01 {
            self.accuracy > other.accuracy
        } else {
            self.f1_score > other.f1_score
        }
    }
}

impl Default for ModelQualityMetrics {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0)
    }
}

/// Quality thresholds for model acceptance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum accuracy required
    pub min_accuracy: f64,
    /// Minimum precision required
    pub min_precision: f64,
    /// Minimum recall required
    pub min_recall: f64,
    /// Minimum F1 score required
    pub min_f1: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_accuracy: 0.85,
            min_precision: 0.80,
            min_recall: 0.80,
            min_f1: 0.80,
        }
    }
}

/// A versioned model entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Version identifier
    pub version: ModelVersion,
    /// Quality metrics at release
    pub metrics: ModelQualityMetrics,
    /// Release timestamp (Unix millis)
    pub released_at: u64,
    /// Optional description
    pub description: String,
    /// Model artifact path/identifier
    pub artifact_path: String,
    /// Is this the current active version?
    pub is_active: bool,
    /// Was this version rolled back?
    pub rolled_back: bool,
    /// Rollback reason (if applicable)
    pub rollback_reason: Option<String>,
}

impl ModelEntry {
    /// Create a new model entry.
    pub fn new(
        version: ModelVersion,
        metrics: ModelQualityMetrics,
        description: impl Into<String>,
        artifact_path: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            version,
            metrics,
            released_at: now,
            description: description.into(),
            artifact_path: artifact_path.into(),
            is_active: false,
            rolled_back: false,
            rollback_reason: None,
        }
    }
}

/// Result of a rollback operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    /// Whether rollback succeeded
    pub success: bool,
    /// Previous version (rolled back from)
    pub from_version: ModelVersion,
    /// New active version (rolled back to)
    pub to_version: ModelVersion,
    /// Reason for rollback
    pub reason: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Model version manager with rollback capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersionManager {
    /// Version history (newest last)
    versions: VecDeque<ModelEntry>,
    /// Current active version index
    active_index: Option<usize>,
    /// Quality thresholds
    thresholds: QualityThresholds,
    /// Maximum versions to retain
    max_history: usize,
    /// Rollback history
    rollback_history: Vec<RollbackResult>,
}

impl Default for ModelVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelVersionManager {
    /// Create a new version manager.
    pub fn new() -> Self {
        Self {
            versions: VecDeque::new(),
            active_index: None,
            thresholds: QualityThresholds::default(),
            max_history: 10,
            rollback_history: Vec::new(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: QualityThresholds) -> Self {
        Self {
            thresholds,
            ..Self::new()
        }
    }

    /// Set maximum history size.
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max.max(2); // Keep at least 2 for rollback
        self
    }

    /// Get current active version.
    pub fn active_version(&self) -> Option<&ModelEntry> {
        self.active_index.and_then(|i| self.versions.get(i))
    }

    /// Get all versions.
    pub fn versions(&self) -> impl Iterator<Item = &ModelEntry> {
        self.versions.iter()
    }

    /// Get version count.
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Get quality thresholds.
    pub fn thresholds(&self) -> &QualityThresholds {
        &self.thresholds
    }

    /// Register a new model version.
    ///
    /// Returns Ok(true) if version was activated, Ok(false) if registered but not activated
    /// (due to quality issues), or Err if registration failed.
    pub fn register_version(&mut self, mut entry: ModelEntry) -> Result<bool, String> {
        // Validate version is newer
        if let Some(latest) = self.versions.back() {
            if entry.version <= latest.version {
                return Err(format!(
                    "Version {} must be greater than current {}",
                    entry.version, latest.version
                ));
            }
        }

        // Check quality thresholds
        let meets_quality = entry.metrics.meets_thresholds(&self.thresholds);

        // Check if better than current active
        let is_better = self
            .active_version()
            .map(|active| entry.metrics.is_better_than(&active.metrics))
            .unwrap_or(true);

        // Decide whether to activate
        let should_activate = meets_quality && is_better;

        if should_activate {
            // Deactivate current
            if let Some(idx) = self.active_index {
                if let Some(current) = self.versions.get_mut(idx) {
                    current.is_active = false;
                }
            }

            // Activate new
            entry.is_active = true;
            self.versions.push_back(entry);
            self.active_index = Some(self.versions.len() - 1);
        } else {
            // Register but don't activate
            entry.is_active = false;
            self.versions.push_back(entry);
        }

        // Prune old versions
        self.prune_history();

        Ok(should_activate)
    }

    /// Rollback to the previous version.
    pub fn rollback(&mut self, reason: impl Into<String>) -> Result<RollbackResult, String> {
        let reason = reason.into();

        // Need at least 2 versions to rollback
        if self.versions.len() < 2 {
            return Err("Not enough versions to rollback".to_string());
        }

        let current_idx = self.active_index.ok_or("No active version")?;
        let current_version = self.versions[current_idx].version.clone();

        // Find previous non-rolled-back version
        let prev_idx = self
            .versions
            .iter()
            .enumerate()
            .rev()
            .skip(1) // Skip current
            .find(|(_, e)| !e.rolled_back)
            .map(|(i, _)| i)
            .ok_or("No previous version available for rollback")?;

        let prev_version = self.versions[prev_idx].version.clone();

        // Mark current as rolled back
        if let Some(current) = self.versions.get_mut(current_idx) {
            current.is_active = false;
            current.rolled_back = true;
            current.rollback_reason = Some(reason.clone());
        }

        // Activate previous
        if let Some(prev) = self.versions.get_mut(prev_idx) {
            prev.is_active = true;
        }
        self.active_index = Some(prev_idx);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let result = RollbackResult {
            success: true,
            from_version: current_version,
            to_version: prev_version,
            reason,
            timestamp: now,
        };

        self.rollback_history.push(result.clone());

        Ok(result)
    }

    /// Rollback to a specific version.
    pub fn rollback_to(
        &mut self,
        target: &ModelVersion,
        reason: impl Into<String>,
    ) -> Result<RollbackResult, String> {
        let reason = reason.into();

        let target_idx = self
            .versions
            .iter()
            .position(|e| &e.version == target)
            .ok_or_else(|| format!("Version {} not found", target))?;

        let current_idx = self.active_index.ok_or("No active version")?;

        if target_idx == current_idx {
            return Err("Target is already the active version".to_string());
        }

        let current_version = self.versions[current_idx].version.clone();

        // Mark current as rolled back
        if let Some(current) = self.versions.get_mut(current_idx) {
            current.is_active = false;
            current.rolled_back = true;
            current.rollback_reason = Some(reason.clone());
        }

        // Activate target
        if let Some(target_entry) = self.versions.get_mut(target_idx) {
            target_entry.is_active = true;
            target_entry.rolled_back = false; // Clear previous rollback if any
        }
        self.active_index = Some(target_idx);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let result = RollbackResult {
            success: true,
            from_version: current_version,
            to_version: target.clone(),
            reason,
            timestamp: now,
        };

        self.rollback_history.push(result.clone());

        Ok(result)
    }

    /// Get rollback history.
    pub fn rollback_history(&self) -> &[RollbackResult] {
        &self.rollback_history
    }

    /// Check if current model needs rollback based on new metrics.
    ///
    /// Implements Jidoka (stop the line) principle.
    pub fn check_quality(&self, current_metrics: &ModelQualityMetrics) -> Option<String> {
        if !current_metrics.meets_thresholds(&self.thresholds) {
            return Some(format!(
                "Quality below thresholds: accuracy={:.2} (min={:.2})",
                current_metrics.accuracy, self.thresholds.min_accuracy
            ));
        }

        // Compare with previous version (should not regress)
        if let Some(active) = self.active_version() {
            if current_metrics.accuracy < active.metrics.accuracy - 0.05 {
                return Some(format!(
                    "Accuracy regression: {:.2} → {:.2} (>5% drop)",
                    active.metrics.accuracy, current_metrics.accuracy
                ));
            }
        }

        None
    }

    /// Auto-rollback if quality check fails.
    pub fn auto_rollback_if_needed(
        &mut self,
        current_metrics: &ModelQualityMetrics,
    ) -> Option<RollbackResult> {
        if let Some(reason) = self.check_quality(current_metrics) {
            self.rollback(format!("Auto-rollback: {}", reason)).ok()
        } else {
            None
        }
    }

    /// Generate markdown report.
    pub fn to_markdown(&self) -> String {
        let mut report = String::from("## Model Version Report\n\n");

        // Active version
        if let Some(active) = self.active_version() {
            report.push_str(&format!(
                "**Active Version**: {} | Accuracy: {:.1}% | F1: {:.3}\n\n",
                active.version,
                active.metrics.accuracy * 100.0,
                active.metrics.f1_score
            ));
        } else {
            report.push_str("**Active Version**: None\n\n");
        }

        // Version history
        report.push_str("### Version History\n\n");
        report.push_str("| Version | Accuracy | F1 | Status | Released |\n");
        report.push_str("|---------|----------|----|---------|---------|\n");

        for entry in self.versions.iter().rev() {
            let status = if entry.is_active {
                "✅ Active"
            } else if entry.rolled_back {
                "🔙 Rolled Back"
            } else {
                "📦 Available"
            };

            // Format timestamp
            let released = chrono_lite_format(entry.released_at);

            report.push_str(&format!(
                "| {} | {:.1}% | {:.3} | {} | {} |\n",
                entry.version,
                entry.metrics.accuracy * 100.0,
                entry.metrics.f1_score,
                status,
                released
            ));
        }

        // Rollback history
        if !self.rollback_history.is_empty() {
            report.push_str("\n### Rollback History\n\n");
            for rb in &self.rollback_history {
                report.push_str(&format!(
                    "- {} → {}: {}\n",
                    rb.from_version, rb.to_version, rb.reason
                ));
            }
        }

        report
    }

    fn prune_history(&mut self) {
        while self.versions.len() > self.max_history {
            // Don't remove active version
            if self.active_index == Some(0) {
                break;
            }

            self.versions.pop_front();

            // Adjust active index
            if let Some(idx) = self.active_index {
                if idx > 0 {
                    self.active_index = Some(idx - 1);
                }
            }
        }
    }
}

/// Simple timestamp formatter (no chrono dependency).
fn chrono_lite_format(millis: u64) -> String {
    // Simple: just show as relative or ISO-ish
    let secs = millis / 1000;
    let days = secs / 86400;
    if days > 0 {
        format!("{}d ago", days)
    } else {
        let hours = secs / 3600;
        if hours > 0 {
            format!("{}h ago", hours)
        } else {
            "recent".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ModelVersion tests
    // ========================================================================

    #[test]
    fn model_version_new() {
        let v = ModelVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn model_version_display() {
        let v = ModelVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "v1.2.3");
    }

    #[test]
    fn model_version_parse() {
        assert_eq!(
            ModelVersion::parse("1.2.3"),
            Some(ModelVersion::new(1, 2, 3))
        );
        assert_eq!(
            ModelVersion::parse("v1.2.3"),
            Some(ModelVersion::new(1, 2, 3))
        );
        assert_eq!(ModelVersion::parse("invalid"), None);
        assert_eq!(ModelVersion::parse("1.2"), None);
    }

    #[test]
    fn model_version_bump() {
        let v = ModelVersion::new(1, 2, 3);

        assert_eq!(v.bump_major(), ModelVersion::new(2, 0, 0));
        assert_eq!(v.bump_minor(), ModelVersion::new(1, 3, 0));
        assert_eq!(v.bump_patch(), ModelVersion::new(1, 2, 4));
    }

    #[test]
    fn model_version_ordering() {
        let v1 = ModelVersion::new(1, 0, 0);
        let v2 = ModelVersion::new(1, 1, 0);
        let v3 = ModelVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    // ========================================================================
    // ModelQualityMetrics tests
    // ========================================================================

    #[test]
    fn quality_metrics_new() {
        let m = ModelQualityMetrics::new(0.9, 0.85, 0.88, 0.86, 0.75, 0.2, 1000);
        assert!((m.accuracy - 0.9).abs() < 0.001);
        assert_eq!(m.sample_count, 1000);
    }

    #[test]
    fn quality_metrics_clamps() {
        let m = ModelQualityMetrics::new(1.5, -0.1, 0.5, 0.5, 0.5, 0.5, 100);
        assert!((m.accuracy - 1.0).abs() < 0.001);
        assert!((m.precision - 0.0).abs() < 0.001);
    }

    #[test]
    fn quality_metrics_meets_thresholds() {
        let thresholds = QualityThresholds::default();

        // Good metrics
        let good = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        assert!(good.meets_thresholds(&thresholds));

        // Bad accuracy
        let bad = ModelQualityMetrics::new(0.70, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        assert!(!bad.meets_thresholds(&thresholds));
    }

    #[test]
    fn quality_metrics_is_better_than() {
        let m1 = ModelQualityMetrics::new(0.85, 0.80, 0.80, 0.80, 0.7, 0.3, 1000);
        let m2 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);

        assert!(m2.is_better_than(&m1));
        assert!(!m1.is_better_than(&m2));
    }

    // ========================================================================
    // ModelVersionManager tests
    // ========================================================================

    #[test]
    fn version_manager_new() {
        let mgr = ModelVersionManager::new();
        assert_eq!(mgr.version_count(), 0);
        assert!(mgr.active_version().is_none());
    }

    #[test]
    fn version_manager_register_first() {
        let mut mgr = ModelVersionManager::new();

        let metrics = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let entry = ModelEntry::new(
            ModelVersion::new(1, 0, 0),
            metrics,
            "Initial version",
            "/models/v1.0.0.bin",
        );

        let result = mgr.register_version(entry);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be activated

        assert_eq!(mgr.version_count(), 1);
        assert!(mgr.active_version().is_some());
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");
    }

    #[test]
    fn version_manager_register_better_version() {
        let mut mgr = ModelVersionManager::new();

        // Register v1.0.0
        let m1 = ModelQualityMetrics::new(0.85, 0.80, 0.80, 0.80, 0.7, 0.3, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // Register better v1.1.0
        let m2 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        let activated = mgr.register_version(e2).unwrap();

        assert!(activated); // Should activate better version
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.1.0");
    }

    #[test]
    fn version_manager_register_worse_version_not_activated() {
        let mut mgr = ModelVersionManager::new();

        // Register v1.0.0 with good metrics
        let m1 = ModelQualityMetrics::new(0.92, 0.88, 0.88, 0.88, 0.85, 0.15, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // Register worse v1.1.0
        let m2 = ModelQualityMetrics::new(0.86, 0.82, 0.82, 0.82, 0.75, 0.25, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        let activated = mgr.register_version(e2).unwrap();

        assert!(!activated); // Should NOT activate worse version
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");
        assert_eq!(mgr.version_count(), 2); // But still registered
    }

    #[test]
    fn version_manager_register_below_threshold() {
        let mut mgr = ModelVersionManager::new();

        // Register version below threshold
        let m1 = ModelQualityMetrics::new(0.70, 0.65, 0.65, 0.65, 0.5, 0.5, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        let activated = mgr.register_version(e1).unwrap();

        assert!(!activated); // Below threshold, not activated
    }

    #[test]
    fn version_manager_reject_older_version() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 1, 0), m1.clone(), "v1.1", "/v1.1");
        mgr.register_version(e1).unwrap();

        // Try to register older version
        let e2 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1.0", "/v1.0");
        let result = mgr.register_version(e2);

        assert!(result.is_err());
    }

    #[test]
    fn version_manager_rollback() {
        let mut mgr = ModelVersionManager::new();

        // Register v1.0.0
        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // Register v1.1.0
        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.1.0");

        // Rollback
        let result = mgr.rollback("Quality regression detected");
        assert!(result.is_ok());

        let rb = result.unwrap();
        assert!(rb.success);
        assert_eq!(rb.from_version.to_string(), "v1.1.0");
        assert_eq!(rb.to_version.to_string(), "v1.0.0");

        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");
    }

    #[test]
    fn version_manager_rollback_not_enough_versions() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let result = mgr.rollback("Test");
        assert!(result.is_err());
    }

    #[test]
    fn version_manager_rollback_to_specific() {
        let mut mgr = ModelVersionManager::new();

        // Register 3 versions
        for i in 0..3 {
            let m = ModelQualityMetrics::new(
                0.85 + (i as f64 * 0.02),
                0.85,
                0.85,
                0.85,
                0.8,
                0.2,
                1000,
            );
            let e = ModelEntry::new(
                ModelVersion::new(1, i, 0),
                m,
                format!("v1.{}.0", i),
                format!("/v1.{}.0", i),
            );
            mgr.register_version(e).unwrap();
        }

        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.2.0");

        // Rollback to v1.0.0
        let result = mgr.rollback_to(&ModelVersion::new(1, 0, 0), "Rollback to baseline");
        assert!(result.is_ok());

        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");
    }

    #[test]
    fn version_manager_check_quality() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // Good metrics - no issue
        let good = ModelQualityMetrics::new(0.88, 0.83, 0.83, 0.83, 0.78, 0.22, 500);
        assert!(mgr.check_quality(&good).is_none());

        // Bad metrics - below threshold
        let bad = ModelQualityMetrics::new(0.70, 0.65, 0.65, 0.65, 0.5, 0.5, 500);
        assert!(mgr.check_quality(&bad).is_some());

        // Regression - accuracy dropped >5%
        let regressed = ModelQualityMetrics::new(0.84, 0.80, 0.80, 0.80, 0.75, 0.25, 500);
        assert!(mgr.check_quality(&regressed).is_some());
    }

    #[test]
    fn version_manager_auto_rollback() {
        let mut mgr = ModelVersionManager::new();

        // Register two versions
        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        // Simulate degraded metrics
        let degraded = ModelQualityMetrics::new(0.70, 0.65, 0.65, 0.65, 0.5, 0.5, 500);
        let rollback = mgr.auto_rollback_if_needed(&degraded);

        assert!(rollback.is_some());
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");
    }

    #[test]
    fn version_manager_to_markdown() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "Initial", "/v1");
        mgr.register_version(e1).unwrap();

        let md = mgr.to_markdown();
        assert!(md.contains("Model Version Report"));
        assert!(md.contains("v1.0.0"));
        assert!(md.contains("Active"));
    }

    #[test]
    fn version_manager_prune_history() {
        let mut mgr = ModelVersionManager::new().with_max_history(3);

        // Register 5 versions
        for i in 0..5 {
            let m = ModelQualityMetrics::new(
                0.85 + (i as f64 * 0.01),
                0.85,
                0.85,
                0.85,
                0.8,
                0.2,
                1000,
            );
            let e = ModelEntry::new(
                ModelVersion::new(1, i, 0),
                m,
                format!("v1.{}.0", i),
                format!("/v1.{}.0", i),
            );
            mgr.register_version(e).unwrap();
        }

        // Should only have 3 versions
        assert_eq!(mgr.version_count(), 3);
        // Active should still be latest
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.4.0");
    }

    // ========================================================================
    // RollbackResult tests
    // ========================================================================

    #[test]
    fn rollback_history_recorded() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        mgr.rollback("Test rollback").unwrap();

        assert_eq!(mgr.rollback_history().len(), 1);
        assert_eq!(mgr.rollback_history()[0].reason, "Test rollback");
    }

    #[test]
    fn version_manager_default_trait() {
        let mgr = ModelVersionManager::default();
        assert!(mgr.active_version().is_none());
        assert_eq!(mgr.version_count(), 0);
    }

    // ========================================================================
    // Additional coverage: ModelVersion::default and parse edge cases
    // ========================================================================

    #[test]
    fn model_version_default() {
        let v = ModelVersion::default();
        assert_eq!(v, ModelVersion::new(1, 0, 0));
        assert_eq!(v.to_string(), "v1.0.0");
    }

    #[test]
    fn model_version_parse_non_numeric() {
        assert_eq!(ModelVersion::parse("a.b.c"), None);
        assert_eq!(ModelVersion::parse("1.2.x"), None);
        assert_eq!(ModelVersion::parse("1.x.3"), None);
    }

    #[test]
    fn model_version_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ModelVersion::new(1, 0, 0));
        set.insert(ModelVersion::new(1, 0, 0)); // duplicate
        assert_eq!(set.len(), 1);
        set.insert(ModelVersion::new(1, 0, 1));
        assert_eq!(set.len(), 2);
    }

    // ========================================================================
    // Additional coverage: ModelQualityMetrics defaults and edge cases
    // ========================================================================

    #[test]
    fn quality_metrics_default() {
        let m = ModelQualityMetrics::default();
        assert!((m.accuracy - 0.0).abs() < 0.001);
        assert!((m.precision - 0.0).abs() < 0.001);
        assert!((m.recall - 0.0).abs() < 0.001);
        assert!((m.f1_score - 0.0).abs() < 0.001);
        assert!((m.avg_confidence - 0.0).abs() < 0.001);
        assert!((m.fallback_rate - 1.0).abs() < 0.001);
        assert_eq!(m.sample_count, 0);
    }

    #[test]
    fn quality_metrics_is_better_than_f1_tiebreaker() {
        // Accuracy within 0.01 — should use F1 tiebreaker
        let m1 = ModelQualityMetrics::new(0.900, 0.80, 0.80, 0.85, 0.7, 0.3, 1000);
        let m2 = ModelQualityMetrics::new(0.905, 0.85, 0.85, 0.90, 0.8, 0.2, 1000);

        // Accuracy diff = 0.005 <= 0.01, so F1 decides
        assert!(m2.is_better_than(&m1)); // m2 has higher F1
        assert!(!m1.is_better_than(&m2)); // m1 has lower F1
    }

    #[test]
    fn quality_metrics_is_better_than_equal_accuracy_equal_f1() {
        let m1 = ModelQualityMetrics::new(0.90, 0.80, 0.80, 0.85, 0.7, 0.3, 1000);
        let m2 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);

        // Same accuracy, same F1 — neither is better
        assert!(!m1.is_better_than(&m2));
        assert!(!m2.is_better_than(&m1));
    }

    #[test]
    fn quality_metrics_meets_thresholds_all_boundaries() {
        let t = QualityThresholds {
            min_accuracy: 0.85,
            min_precision: 0.80,
            min_recall: 0.80,
            min_f1: 0.80,
        };

        // Exactly at thresholds — should pass
        let exact = ModelQualityMetrics::new(0.85, 0.80, 0.80, 0.80, 0.5, 0.5, 100);
        assert!(exact.meets_thresholds(&t));

        // Below precision
        let bad_prec = ModelQualityMetrics::new(0.90, 0.79, 0.85, 0.85, 0.5, 0.5, 100);
        assert!(!bad_prec.meets_thresholds(&t));

        // Below recall
        let bad_recall = ModelQualityMetrics::new(0.90, 0.85, 0.79, 0.85, 0.5, 0.5, 100);
        assert!(!bad_recall.meets_thresholds(&t));

        // Below F1
        let bad_f1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.79, 0.5, 0.5, 100);
        assert!(!bad_f1.meets_thresholds(&t));
    }

    // ========================================================================
    // Additional coverage: rollback error paths
    // ========================================================================

    #[test]
    fn rollback_no_active_version() {
        let mut mgr = ModelVersionManager::new();

        // Register 2 versions below threshold (neither activated)
        let m1 = ModelQualityMetrics::new(0.70, 0.65, 0.65, 0.65, 0.5, 0.5, 100);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.71, 0.66, 0.66, 0.66, 0.5, 0.5, 100);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        assert_eq!(mgr.version_count(), 2);
        assert!(mgr.active_version().is_none());

        let result = mgr.rollback("No active version");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No active version"));
    }

    #[test]
    fn rollback_to_not_found() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let result = mgr.rollback_to(&ModelVersion::new(9, 9, 9), "Not found");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn rollback_to_already_active() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let result = mgr.rollback_to(&ModelVersion::new(1, 0, 0), "Already active");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already the active version"));
    }

    #[test]
    fn rollback_to_no_active_version() {
        let mut mgr = ModelVersionManager::new();

        // Register below threshold
        let m1 = ModelQualityMetrics::new(0.70, 0.65, 0.65, 0.65, 0.5, 0.5, 100);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let result = mgr.rollback_to(&ModelVersion::new(1, 0, 0), "No active");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No active version"));
    }

    // ========================================================================
    // Additional coverage: to_markdown with all status variants + rollback history
    // ========================================================================

    #[test]
    fn to_markdown_no_active_version() {
        let mgr = ModelVersionManager::new();
        let md = mgr.to_markdown();
        assert!(md.contains("**Active Version**: None"));
    }

    #[test]
    fn to_markdown_with_rolled_back_and_available_status() {
        let mut mgr = ModelVersionManager::new();

        // Register 3 versions
        let m1 = ModelQualityMetrics::new(0.86, 0.82, 0.82, 0.82, 0.7, 0.3, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.88, 0.84, 0.84, 0.84, 0.75, 0.25, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        let m3 = ModelQualityMetrics::new(0.90, 0.86, 0.86, 0.86, 0.8, 0.2, 1000);
        let e3 = ModelEntry::new(ModelVersion::new(1, 2, 0), m3, "v1.2", "/v1.2");
        mgr.register_version(e3).unwrap();

        // Rollback to v1.0.0 — marks v1.2.0 as rolled back
        mgr.rollback_to(&ModelVersion::new(1, 0, 0), "Quality issue").unwrap();

        let md = mgr.to_markdown();
        // v1.0.0 should be Active, v1.2.0 should be Rolled Back, v1.1.0 should be Available
        assert!(md.contains("Active"));
        assert!(md.contains("Rolled Back"));
        assert!(md.contains("Available"));
        // Rollback history section
        assert!(md.contains("Rollback History"));
        assert!(md.contains("Quality issue"));
    }

    // ========================================================================
    // Additional coverage: prune_history with active at index 0
    // ========================================================================

    #[test]
    fn prune_history_active_at_index_zero_breaks() {
        let mut mgr = ModelVersionManager::new().with_max_history(2);

        // Register v1.0.0 (activated)
        let m1 = ModelQualityMetrics::new(0.92, 0.88, 0.88, 0.88, 0.85, 0.15, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // Register v1.1.0 (activated, better)
        let m2 = ModelQualityMetrics::new(0.94, 0.90, 0.90, 0.90, 0.87, 0.13, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        // Rollback to v1.0.0 (now active is at index 0)
        mgr.rollback("test").unwrap();
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.0.0");

        // Register v1.2.0 (not activated because worse than v1.0.0=0.92)
        let m3 = ModelQualityMetrics::new(0.86, 0.82, 0.82, 0.82, 0.75, 0.25, 1000);
        let e3 = ModelEntry::new(ModelVersion::new(1, 2, 0), m3, "v1.2", "/v1.2");
        mgr.register_version(e3).unwrap();

        // Now we have 3 versions but max_history=2
        // prune should break because active_index == Some(0)
        assert!(mgr.version_count() >= 2);
        // Active is still valid
        assert!(mgr.active_version().is_some());
    }

    // ========================================================================
    // Additional coverage: chrono_lite_format branches
    // ========================================================================

    #[test]
    fn chrono_lite_format_recent() {
        let result = chrono_lite_format(0);
        assert_eq!(result, "recent");
    }

    #[test]
    fn chrono_lite_format_hours() {
        // 2 hours in millis
        let two_hours_ms = 2 * 3600 * 1000;
        let result = chrono_lite_format(two_hours_ms);
        assert_eq!(result, "2h ago");
    }

    #[test]
    fn chrono_lite_format_days() {
        // 3 days in millis
        let three_days_ms = 3 * 86400 * 1000;
        let result = chrono_lite_format(three_days_ms);
        assert_eq!(result, "3d ago");
    }

    // ========================================================================
    // Additional coverage: with_thresholds and accessors
    // ========================================================================

    #[test]
    fn version_manager_with_thresholds() {
        let t = QualityThresholds {
            min_accuracy: 0.95,
            min_precision: 0.90,
            min_recall: 0.90,
            min_f1: 0.90,
        };
        let mgr = ModelVersionManager::with_thresholds(t);
        assert!((mgr.thresholds().min_accuracy - 0.95).abs() < 0.001);
        assert!((mgr.thresholds().min_precision - 0.90).abs() < 0.001);
    }

    #[test]
    fn version_manager_with_max_history_minimum() {
        // Setting max_history to 1 should be clamped to 2
        let mgr = ModelVersionManager::new().with_max_history(1);
        // Register 3 versions — should keep at least 2
        let mut mgr = mgr;
        for i in 0..3 {
            let m = ModelQualityMetrics::new(
                0.85 + (i as f64 * 0.02),
                0.85,
                0.85,
                0.85,
                0.8,
                0.2,
                1000,
            );
            let e = ModelEntry::new(
                ModelVersion::new(1, i, 0),
                m,
                format!("v1.{}", i),
                format!("/v1.{}", i),
            );
            mgr.register_version(e).unwrap();
        }
        assert_eq!(mgr.version_count(), 2);
    }

    #[test]
    fn version_manager_versions_iterator() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        let versions: Vec<_> = mgr.versions().collect();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version.to_string(), "v1.0.0");
        assert_eq!(versions[1].version.to_string(), "v1.1.0");
    }

    // ========================================================================
    // Additional coverage: auto_rollback_if_needed with good metrics
    // ========================================================================

    #[test]
    fn auto_rollback_not_needed_with_good_metrics() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        let good = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 500);
        let result = mgr.auto_rollback_if_needed(&good);
        assert!(result.is_none());
        assert_eq!(mgr.active_version().unwrap().version.to_string(), "v1.1.0");
    }

    // ========================================================================
    // Additional coverage: check_quality accuracy regression message
    // ========================================================================

    #[test]
    fn check_quality_accuracy_regression_message() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.95, 0.90, 0.90, 0.90, 0.85, 0.15, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        // >5% regression: 0.95 → 0.88 = 7% drop (but 0.88 still meets default threshold 0.85)
        let regressed = ModelQualityMetrics::new(0.88, 0.85, 0.85, 0.85, 0.8, 0.2, 500);
        let reason = mgr.check_quality(&regressed);
        assert!(reason.is_some());
        let msg = reason.unwrap();
        assert!(msg.contains("Accuracy regression"));
        assert!(msg.contains(">5% drop"));
    }

    #[test]
    fn check_quality_no_active_version() {
        let mgr = ModelVersionManager::new();
        // No active version, but metrics meet threshold
        let good = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 500);
        assert!(mgr.check_quality(&good).is_none());
    }

    // ========================================================================
    // Additional coverage: ModelEntry fields
    // ========================================================================

    #[test]
    fn model_entry_new_defaults() {
        let m = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let entry = ModelEntry::new(ModelVersion::new(1, 0, 0), m, "test", "/path");

        assert_eq!(entry.description, "test");
        assert_eq!(entry.artifact_path, "/path");
        assert!(!entry.is_active);
        assert!(!entry.rolled_back);
        assert!(entry.rollback_reason.is_none());
        assert!(entry.released_at > 0);
    }

    // ========================================================================
    // Additional coverage: rollback marks entry correctly
    // ========================================================================

    #[test]
    fn rollback_marks_entry_as_rolled_back() {
        let mut mgr = ModelVersionManager::new();

        let m1 = ModelQualityMetrics::new(0.90, 0.85, 0.85, 0.85, 0.8, 0.2, 1000);
        let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
        mgr.register_version(e1).unwrap();

        let m2 = ModelQualityMetrics::new(0.92, 0.87, 0.87, 0.87, 0.82, 0.18, 1000);
        let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v1.1", "/v1.1");
        mgr.register_version(e2).unwrap();

        mgr.rollback("Bad quality").unwrap();

        // v1.1.0 should be marked as rolled back with reason
        let versions: Vec<_> = mgr.versions().collect();
        let v110 = versions.iter().find(|v| v.version.to_string() == "v1.1.0").unwrap();
        assert!(v110.rolled_back);
        assert_eq!(v110.rollback_reason.as_deref(), Some("Bad quality"));
        assert!(!v110.is_active);
    }

    #[test]
    fn rollback_to_clears_previous_rollback_flag() {
        let mut mgr = ModelVersionManager::new();

        // Register 3 versions
        for i in 0..3 {
            let m = ModelQualityMetrics::new(
                0.86 + (i as f64 * 0.02),
                0.82,
                0.82,
                0.82,
                0.75,
                0.25,
                1000,
            );
            let e = ModelEntry::new(
                ModelVersion::new(1, i, 0),
                m,
                format!("v1.{}", i),
                format!("/v1.{}", i),
            );
            mgr.register_version(e).unwrap();
        }

        // Rollback v1.2.0 → v1.0.0
        mgr.rollback_to(&ModelVersion::new(1, 0, 0), "first rollback").unwrap();

        // v1.0.0 should have rolled_back cleared and is_active true
        let active = mgr.active_version().unwrap();
        assert!(!active.rolled_back);
        assert!(active.is_active);

        // Rollback history should have entry
        assert_eq!(mgr.rollback_history().len(), 1);
    }
}
