//! Lock-to-data binding analysis for pthread synchronization (DECY-077).
//!
//! Analyzes C code with pthread_mutex locks to determine which locks
//! protect which data variables, enabling safe Mutex<T> generation.

use decy_hir::{HirExpression, HirFunction, HirStatement};
use std::collections::{HashMap, HashSet};

/// Represents a locked code region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockRegion {
    /// Name of the lock variable
    pub lock_name: String,
    /// Starting statement index (lock call)
    pub start_index: usize,
    /// Ending statement index (unlock call)
    pub end_index: usize,
}

/// Mapping from locks to protected data variables.
#[derive(Debug, Clone)]
pub struct LockDataMapping {
    /// Maps lock name → set of protected variable names
    lock_to_data: HashMap<String, HashSet<String>>,
}

impl LockDataMapping {
    /// Create a new empty mapping.
    pub fn new() -> Self {
        Self {
            lock_to_data: HashMap::new(),
        }
    }

    /// Check if a variable is protected by a specific lock.
    pub fn is_protected_by(&self, data: &str, lock: &str) -> bool {
        self.lock_to_data
            .get(lock)
            .map(|vars| vars.contains(data))
            .unwrap_or(false)
    }

    /// Get all data variables protected by a lock.
    pub fn get_protected_data(&self, lock: &str) -> Vec<String> {
        self.lock_to_data
            .get(lock)
            .map(|vars| vars.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all locks tracked in this mapping.
    pub fn get_locks(&self) -> Vec<String> {
        self.lock_to_data.keys().cloned().collect()
    }

    /// Add a data variable to a lock's protection set.
    fn add_protected_data(&mut self, lock: String, data: String) {
        self.lock_to_data
            .entry(lock)
            .or_default()
            .insert(data);
    }
}

impl Default for LockDataMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes pthread lock usage and protected data.
pub struct LockAnalyzer;

impl LockAnalyzer {
    /// Create a new lock analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Find all locked regions in a function.
    ///
    /// Identifies pthread_mutex_lock/unlock pairs and returns
    /// the code regions they protect.
    pub fn find_lock_regions(&self, func: &HirFunction) -> Vec<LockRegion> {
        let mut regions = Vec::new();
        let body = func.body();

        // Track active locks (lock name -> start index)
        let mut active_locks: HashMap<String, usize> = HashMap::new();

        for (idx, stmt) in body.iter().enumerate() {
            // Check for pthread_mutex_lock calls
            if let Some(lock_name) = Self::extract_lock_call(stmt) {
                active_locks.insert(lock_name, idx);
            }
            // Check for pthread_mutex_unlock calls
            else if let Some(unlock_name) = Self::extract_unlock_call(stmt) {
                if let Some(start_idx) = active_locks.remove(&unlock_name) {
                    regions.push(LockRegion {
                        lock_name: unlock_name,
                        start_index: start_idx,
                        end_index: idx,
                    });
                }
            }
        }

        regions
    }

    /// Extract lock name from pthread_mutex_lock call.
    fn extract_lock_call(stmt: &HirStatement) -> Option<String> {
        if let HirStatement::Expression(HirExpression::FunctionCall {
            function,
            arguments,
        }) = stmt
        {
            if function == "pthread_mutex_lock" {
                // Extract lock name from &lock argument
                if let Some(HirExpression::AddressOf(inner)) = arguments.first() {
                    if let HirExpression::Variable(name) = &**inner {
                        return Some(name.clone());
                    }
                }
            }
        }
        None
    }

    /// Extract lock name from pthread_mutex_unlock call.
    fn extract_unlock_call(stmt: &HirStatement) -> Option<String> {
        if let HirStatement::Expression(HirExpression::FunctionCall {
            function,
            arguments,
        }) = stmt
        {
            if function == "pthread_mutex_unlock" {
                // Extract lock name from &lock argument
                if let Some(HirExpression::AddressOf(inner)) = arguments.first() {
                    if let HirExpression::Variable(name) = &**inner {
                        return Some(name.clone());
                    }
                }
            }
        }
        None
    }

    /// Analyze lock-to-data mapping for a function.
    ///
    /// Determines which locks protect which data variables based
    /// on variable accesses within locked regions.
    pub fn analyze_lock_data_mapping(&self, func: &HirFunction) -> LockDataMapping {
        let mut mapping = LockDataMapping::new();
        let regions = self.find_lock_regions(func);
        let body = func.body();

        // For each lock region, find all accessed variables
        for region in regions {
            let protected_vars = self.find_accessed_variables_in_region(body, &region);
            for var in protected_vars {
                mapping.add_protected_data(region.lock_name.clone(), var);
            }
        }

        mapping
    }

    /// Find all variables accessed in a locked region.
    fn find_accessed_variables_in_region(
        &self,
        body: &[HirStatement],
        region: &LockRegion,
    ) -> HashSet<String> {
        let mut accessed = HashSet::new();

        // Scan statements in the region (excluding lock/unlock calls)
        for idx in (region.start_index + 1)..region.end_index {
            if let Some(stmt) = body.get(idx) {
                self.collect_accessed_variables(stmt, &mut accessed);
            }
        }

        accessed
    }

    /// Recursively collect all variable names accessed in a statement.
    fn collect_accessed_variables(&self, stmt: &HirStatement, accessed: &mut HashSet<String>) {
        match stmt {
            HirStatement::Assignment { target, value } => {
                accessed.insert(target.clone());
                self.collect_variables_from_expr(value, accessed);
            }
            HirStatement::VariableDeclaration {
                initializer: Some(init),
                ..
            } => {
                // Local variable declarations don't count as protected data
                // But if the initializer reads from other variables, those count
                self.collect_variables_from_expr(init, accessed);
                // Don't add the variable name itself - it's local to this scope
            }
            HirStatement::VariableDeclaration {
                initializer: None, ..
            } => {
                // No initializer, nothing to track
            }
            HirStatement::Return(Some(e)) => {
                self.collect_variables_from_expr(e, accessed);
            }
            HirStatement::Return(None) => {
                // No return value, nothing to track
            }
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.collect_variables_from_expr(condition, accessed);
                for s in then_block {
                    self.collect_accessed_variables(s, accessed);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.collect_accessed_variables(s, accessed);
                    }
                }
            }
            HirStatement::While { condition, body } => {
                self.collect_variables_from_expr(condition, accessed);
                for s in body {
                    self.collect_accessed_variables(s, accessed);
                }
            }
            HirStatement::Expression(expr) => {
                self.collect_variables_from_expr(expr, accessed);
            }
            HirStatement::DerefAssignment { target, value } => {
                self.collect_variables_from_expr(target, accessed);
                self.collect_variables_from_expr(value, accessed);
            }
            HirStatement::ArrayIndexAssignment {
                array,
                index,
                value,
            } => {
                self.collect_variables_from_expr(array, accessed);
                self.collect_variables_from_expr(index, accessed);
                self.collect_variables_from_expr(value, accessed);
            }
            HirStatement::FieldAssignment {
                object,
                field: _,
                value,
            } => {
                self.collect_variables_from_expr(object, accessed);
                self.collect_variables_from_expr(value, accessed);
            }
            _ => {
                // Break, Continue, etc. don't access variables
            }
        }
    }

    /// Collect variable names from an expression.
    #[allow(clippy::only_used_in_recursion)]
    fn collect_variables_from_expr(&self, expr: &HirExpression, accessed: &mut HashSet<String>) {
        match expr {
            HirExpression::Variable(name) => {
                accessed.insert(name.clone());
            }
            HirExpression::BinaryOp { left, right, .. } => {
                self.collect_variables_from_expr(left, accessed);
                self.collect_variables_from_expr(right, accessed);
            }
            HirExpression::UnaryOp { operand, .. } => {
                self.collect_variables_from_expr(operand, accessed);
            }
            HirExpression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.collect_variables_from_expr(arg, accessed);
                }
            }
            HirExpression::AddressOf(inner) | HirExpression::Dereference(inner) => {
                self.collect_variables_from_expr(inner, accessed);
            }
            HirExpression::ArrayIndex { array, index } => {
                self.collect_variables_from_expr(array, accessed);
                self.collect_variables_from_expr(index, accessed);
            }
            HirExpression::FieldAccess { object, .. } => {
                self.collect_variables_from_expr(object, accessed);
            }
            HirExpression::Cast { expr, .. } => {
                self.collect_variables_from_expr(expr, accessed);
            }
            // Literals and other expressions don't reference variables
            _ => {}
        }
    }

    /// Check for lock discipline violations.
    ///
    /// Detects:
    /// - Locks without unlocks
    /// - Unlocks without locks
    /// - Mismatched lock/unlock pairs
    ///
    /// Returns a list of violation descriptions.
    pub fn check_lock_discipline(&self, func: &HirFunction) -> Vec<String> {
        let mut violations = Vec::new();
        let body = func.body();

        // Track active locks
        let mut active_locks: HashMap<String, usize> = HashMap::new();

        for (idx, stmt) in body.iter().enumerate() {
            // Check for lock calls
            if let Some(lock_name) = Self::extract_lock_call(stmt) {
                active_locks.insert(lock_name, idx);
            }
            // Check for unlock calls
            else if let Some(unlock_name) = Self::extract_unlock_call(stmt) {
                if active_locks.remove(&unlock_name).is_none() {
                    // Unlock without corresponding lock
                    violations.push(format!(
                        "Unlock without lock: pthread_mutex_unlock(&{}) at statement {}",
                        unlock_name, idx
                    ));
                }
            }
        }

        // Check for unmatched locks (locks without unlocks)
        for (lock_name, start_idx) in active_locks {
            violations.push(format!(
                "Unmatched lock: pthread_mutex_lock(&{}) at statement {} has no corresponding unlock",
                lock_name, start_idx
            ));
        }

        violations
    }
}

impl Default for LockAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
