//! Lock discipline verification (DECY-079).
//!
//! Validates that all accesses to shared data are properly protected by locks
//! and detects potential deadlocks.

use decy_analyzer::lock_analysis::LockAnalyzer;
use decy_hir::HirFunction;

/// Comprehensive lock discipline report
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockDisciplineReport {
    /// Number of unprotected data accesses detected
    pub unprotected_accesses: usize,
    /// Number of lock/unlock violations
    pub lock_violations: usize,
    /// Number of deadlock warnings
    pub deadlock_warnings: usize,
}

impl LockDisciplineReport {
    /// Check if the code has no lock discipline violations
    pub fn is_clean(&self) -> bool {
        self.unprotected_accesses == 0
            && self.lock_violations == 0
            && self.deadlock_warnings == 0
    }
}

/// Lock discipline checker
pub struct LockDisciplineChecker<'a> {
    analyzer: &'a LockAnalyzer,
}

impl<'a> LockDisciplineChecker<'a> {
    /// Create a new lock discipline checker
    pub fn new(analyzer: &'a LockAnalyzer) -> Self {
        Self { analyzer }
    }

    /// Check for unprotected data accesses
    ///
    /// Detects when shared data (identified by lock analysis) is accessed
    /// outside of locked regions.
    pub fn check_unprotected_access(&self, _func: &HirFunction) -> Vec<String> {
        // RED phase: Stub implementation
        todo!("DECY-079: Implement unprotected data access detection")
    }

    /// Check for potential deadlocks
    ///
    /// Analyzes lock ordering across multiple functions to detect
    /// inconsistent lock acquisition patterns that could cause deadlocks.
    pub fn check_deadlock_risk(&self, _functions: &[HirFunction]) -> Vec<String> {
        // RED phase: Stub implementation
        todo!("DECY-079: Implement deadlock detection")
    }

    /// Comprehensive lock discipline check
    ///
    /// Runs all lock discipline checks and returns a summary report.
    pub fn check_all(&self, func: &HirFunction) -> LockDisciplineReport {
        let unprotected = self.check_unprotected_access(func);
        let lock_violations = self.analyzer.check_lock_discipline(func);

        LockDisciplineReport {
            unprotected_accesses: unprotected.len(),
            lock_violations: lock_violations.len(),
            deadlock_warnings: 0, // Single function can't have cross-function deadlocks
        }
    }
}
