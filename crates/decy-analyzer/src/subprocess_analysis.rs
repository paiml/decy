//! Fork/exec subprocess pattern detection for C-to-Rust transpilation.
//!
//! Detects C subprocess patterns like fork()+exec*() and transforms them
//! to Rust's `std::process::Command` API.

use decy_hir::{HirExpression, HirFunction, HirStatement};

/// Detected fork/exec subprocess pattern.
#[derive(Debug, Clone, Default)]
pub struct ForkExecPattern {
    /// Whether fork() was detected
    pub has_fork: bool,
    /// Whether an exec*() call was detected
    pub has_exec: bool,
    /// Whether wait*() was detected
    pub has_wait: bool,
    /// Command path (first arg to exec*)
    pub command: Option<String>,
    /// Arguments extracted from exec*()
    pub args: Vec<String>,
    /// Variable holding fork() result
    pub pid_var: Option<String>,
}

/// Detects fork/exec subprocess patterns in HIR functions.
pub struct SubprocessDetector {
    /// Exec function variants to detect
    exec_functions: Vec<&'static str>,
    /// Wait function variants to detect
    wait_functions: Vec<&'static str>,
}

impl SubprocessDetector {
    /// Create a new subprocess detector.
    pub fn new() -> Self {
        Self {
            exec_functions: vec![
                "execl", "execlp", "execle", "execv", "execvp", "execve", "execvpe",
            ],
            wait_functions: vec!["wait", "waitpid", "wait3", "wait4"],
        }
    }

    /// Detect fork/exec patterns in a function.
    pub fn detect(&self, func: &HirFunction) -> Vec<ForkExecPattern> {
        let mut patterns = Vec::new();
        let mut current = ForkExecPattern::default();

        self.analyze_statements(func.body(), &mut current);

        if current.has_fork || current.has_exec || current.has_wait {
            patterns.push(current);
        }

        patterns
    }

    fn analyze_statements(&self, stmts: &[HirStatement], pattern: &mut ForkExecPattern) {
        for stmt in stmts {
            self.analyze_statement(stmt, pattern);
        }
    }

    fn analyze_statement(&self, stmt: &HirStatement, pattern: &mut ForkExecPattern) {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                initializer: Some(init),
                ..
            } => {
                if self.is_fork_call(init) {
                    pattern.has_fork = true;
                    pattern.pid_var = Some(name.clone());
                }
                self.analyze_expression(init, pattern);
            }
            HirStatement::Expression(expr) => {
                self.analyze_expression(expr, pattern);
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                self.analyze_statements(then_block, pattern);
                if let Some(else_stmts) = else_block {
                    self.analyze_statements(else_stmts, pattern);
                }
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                self.analyze_statements(body, pattern);
            }
            _ => {}
        }
    }

    fn analyze_expression(&self, expr: &HirExpression, pattern: &mut ForkExecPattern) {
        if let HirExpression::FunctionCall {
            function,
            arguments,
        } = expr
        {
            if function == "fork" {
                pattern.has_fork = true;
            } else if self.exec_functions.contains(&function.as_str()) {
                pattern.has_exec = true;
                self.extract_exec_args(arguments, pattern);
            } else if self.wait_functions.contains(&function.as_str()) {
                pattern.has_wait = true;
            }
        }
    }

    fn is_fork_call(&self, expr: &HirExpression) -> bool {
        matches!(
            expr,
            HirExpression::FunctionCall { function, .. } if function == "fork"
        )
    }

    fn extract_exec_args(&self, args: &[HirExpression], pattern: &mut ForkExecPattern) {
        for (i, arg) in args.iter().enumerate() {
            if let HirExpression::StringLiteral(s) = arg {
                if i == 0 {
                    pattern.command = Some(s.clone());
                } else {
                    pattern.args.push(s.clone());
                }
            }
        }
    }
}

impl Default for SubprocessDetector {
    fn default() -> Self {
        Self::new()
    }
}
