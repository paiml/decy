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
    pub fn check_unprotected_access(&self, func: &HirFunction) -> Vec<String> {
        let mut violations = Vec::new();

        // 1. Identify protected data from lock analysis
        let mapping = self.analyzer.analyze_lock_data_mapping(func);
        let protected_vars: std::collections::HashSet<String> = mapping
            .get_locks()
            .iter()
            .flat_map(|lock| mapping.get_protected_data(lock))
            .collect();

        // 2. Find lock regions
        let lock_regions = self.analyzer.find_lock_regions(func);

        // 3. Check all statements outside locked regions
        let body = func.body();
        for (idx, stmt) in body.iter().enumerate() {
            // Skip if this statement is inside a locked region
            if self.is_inside_any_region(idx, &lock_regions) {
                continue;
            }

            // Check if statement accesses protected data
            let accessed_vars = self.collect_accessed_vars(stmt);
            for var in accessed_vars {
                if protected_vars.contains(&var) {
                    violations.push(format!(
                        "Unprotected access to '{}' at statement {} (outside locked region)",
                        var, idx
                    ));
                }
            }
        }

        violations
    }

    /// Check if a statement index is inside any lock region
    fn is_inside_any_region(
        &self,
        idx: usize,
        regions: &[decy_analyzer::lock_analysis::LockRegion],
    ) -> bool {
        regions
            .iter()
            .any(|r| idx > r.start_index && idx < r.end_index)
    }

    /// Collect all variable names accessed in a statement
    fn collect_accessed_vars(&self, stmt: &decy_hir::HirStatement) -> Vec<String> {
        use decy_hir::HirStatement;
        let mut vars = Vec::new();

        match stmt {
            HirStatement::Assignment { target, value } => {
                vars.push(target.clone());
                self.collect_vars_from_expr(value, &mut vars);
            }
            HirStatement::VariableDeclaration {
                initializer: Some(init),
                ..
            } => {
                self.collect_vars_from_expr(init, &mut vars);
            }
            HirStatement::Expression(expr) => {
                self.collect_vars_from_expr(expr, &mut vars);
            }
            HirStatement::Return(Some(expr)) => {
                self.collect_vars_from_expr(expr, &mut vars);
            }
            _ => {}
        }

        vars
    }

    /// Recursively collect variable names from an expression
    fn collect_vars_from_expr(&self, expr: &decy_hir::HirExpression, vars: &mut Vec<String>) {
        use decy_hir::HirExpression;

        match expr {
            HirExpression::Variable(name) => {
                vars.push(name.clone());
            }
            HirExpression::BinaryOp { left, right, .. } => {
                self.collect_vars_from_expr(left, vars);
                self.collect_vars_from_expr(right, vars);
            }
            HirExpression::UnaryOp { operand, .. } => {
                self.collect_vars_from_expr(operand, vars);
            }
            HirExpression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.collect_vars_from_expr(arg, vars);
                }
            }
            HirExpression::AddressOf(inner) | HirExpression::Dereference(inner) => {
                self.collect_vars_from_expr(inner, vars);
            }
            HirExpression::ArrayIndex { array, index } => {
                self.collect_vars_from_expr(array, vars);
                self.collect_vars_from_expr(index, vars);
            }
            HirExpression::FieldAccess { object, .. } => {
                self.collect_vars_from_expr(object, vars);
            }
            HirExpression::Cast { expr, .. } => {
                self.collect_vars_from_expr(expr, vars);
            }
            _ => {}
        }
    }

    /// Check for potential deadlocks
    ///
    /// Analyzes lock ordering across multiple functions to detect
    /// inconsistent lock acquisition patterns that could cause deadlocks.
    pub fn check_deadlock_risk(&self, functions: &[HirFunction]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Edge case: single lock or no locks can't deadlock
        if functions.is_empty() {
            return warnings;
        }

        // 1. Extract lock ordering for each function
        let mut lock_orderings: Vec<Vec<String>> = Vec::new();
        for func in functions {
            let ordering = self.extract_lock_ordering(func);
            if !ordering.is_empty() {
                lock_orderings.push(ordering);
            }
        }

        // 2. Check for inconsistent orderings
        for i in 0..lock_orderings.len() {
            for j in (i + 1)..lock_orderings.len() {
                if let Some(warning) =
                    self.detect_ordering_conflict(&lock_orderings[i], &lock_orderings[j])
                {
                    warnings.push(warning);
                }
            }
        }

        warnings
    }

    /// Extract the lock acquisition order from a function
    fn extract_lock_ordering(&self, func: &HirFunction) -> Vec<String> {
        use decy_hir::{HirExpression, HirStatement};
        let mut ordering = Vec::new();
        let body = func.body();

        for stmt in body {
            if let HirStatement::Expression(HirExpression::FunctionCall {
                function,
                arguments,
            }) = stmt
            {
                if function == "pthread_mutex_lock" {
                    if let Some(HirExpression::AddressOf(inner)) = arguments.first() {
                        if let HirExpression::Variable(name) = &**inner {
                            ordering.push(name.clone());
                        }
                    }
                }
            }
        }

        ordering
    }

    /// Detect if two lock orderings conflict (could cause deadlock)
    fn detect_ordering_conflict(
        &self,
        ordering1: &[String],
        ordering2: &[String],
    ) -> Option<String> {
        // For each pair of locks in ordering1, check if they appear in reverse order in ordering2
        for i in 0..ordering1.len() {
            for j in (i + 1)..ordering1.len() {
                let lock_a = &ordering1[i];
                let lock_b = &ordering1[j];

                // Check if ordering2 has lock_b before lock_a
                let pos_a_in_2 = ordering2.iter().position(|l| l == lock_a);
                let pos_b_in_2 = ordering2.iter().position(|l| l == lock_b);

                if let (Some(pos_a), Some(pos_b)) = (pos_a_in_2, pos_b_in_2) {
                    if pos_b < pos_a {
                        // Found reverse ordering - potential deadlock!
                        return Some(format!(
                            "Potential deadlock: Inconsistent lock ordering detected. \
                             One function acquires {} then {}, another acquires {} then {}",
                            lock_a, lock_b, lock_b, lock_a
                        ));
                    }
                }
            }
        }

        None
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
