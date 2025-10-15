//! Scope-based lifetime analysis for C-to-Rust conversion.
//!
//! This module analyzes C variable scopes to infer Rust lifetimes,
//! tracking variable lifetimes and detecting dangling pointer issues.

use decy_hir::{HirFunction, HirStatement};
use std::collections::HashMap;

/// Represents a scope in the program.
///
/// Scopes are nested (function scope contains statement scopes, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// Unique scope ID
    pub id: usize,
    /// Parent scope ID (None for function scope)
    pub parent: Option<usize>,
    /// Variables declared in this scope
    pub variables: Vec<String>,
    /// Statement indices that belong to this scope
    pub statement_range: (usize, usize),
}

/// Represents the lifetime of a variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableLifetime {
    /// Variable name
    pub name: String,
    /// Scope where variable is declared
    pub declared_in_scope: usize,
    /// First use (statement index)
    pub first_use: usize,
    /// Last use (statement index)
    pub last_use: usize,
    /// Whether variable is returned from function
    pub escapes: bool,
}

/// Scope tree representing nested scopes in a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeTree {
    /// All scopes in the function
    scopes: Vec<Scope>,
    /// Next scope ID to assign
    next_id: usize,
}

impl ScopeTree {
    /// Create a new scope tree with a root function scope.
    pub fn new() -> Self {
        let root = Scope {
            id: 0,
            parent: None,
            variables: Vec::new(),
            statement_range: (0, usize::MAX),
        };

        Self {
            scopes: vec![root],
            next_id: 1,
        }
    }

    /// Add a new scope as a child of the given parent.
    pub fn add_scope(&mut self, parent_id: usize, statement_range: (usize, usize)) -> usize {
        let scope_id = self.next_id;
        self.next_id += 1;

        let scope = Scope {
            id: scope_id,
            parent: Some(parent_id),
            variables: Vec::new(),
            statement_range,
        };

        self.scopes.push(scope);
        scope_id
    }

    /// Add a variable to a scope.
    pub fn add_variable(&mut self, scope_id: usize, var_name: String) {
        if let Some(scope) = self.scopes.iter_mut().find(|s| s.id == scope_id) {
            scope.variables.push(var_name);
        }
    }

    /// Get a scope by ID.
    pub fn get_scope(&self, scope_id: usize) -> Option<&Scope> {
        self.scopes.iter().find(|s| s.id == scope_id)
    }

    /// Get all scopes.
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    /// Check if one scope is nested within another.
    pub fn is_nested_in(&self, inner_id: usize, outer_id: usize) -> bool {
        let mut current = inner_id;
        while let Some(scope) = self.get_scope(current) {
            if scope.id == outer_id {
                return true;
            }
            if let Some(parent_id) = scope.parent {
                current = parent_id;
            } else {
                break;
            }
        }
        false
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Relationship between two lifetimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeRelation {
    /// Lifetimes are identical
    Equal,
    /// First lifetime outlives second
    Outlives,
    /// Second lifetime outlives first
    OutlivedBy,
    /// Lifetimes are independent
    Independent,
}

/// Analyzer for scope-based lifetime inference.
#[derive(Debug)]
pub struct LifetimeAnalyzer;

impl LifetimeAnalyzer {
    /// Create a new lifetime analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Build a scope tree for a function.
    ///
    /// Analyzes the HIR function and creates a tree of nested scopes.
    pub fn build_scope_tree(&self, func: &HirFunction) -> ScopeTree {
        let mut tree = ScopeTree::new();
        let root_scope = 0;

        // Process function body
        self.analyze_statements(func.body(), &mut tree, root_scope, 0);

        tree
    }

    /// Analyze statements and build scope tree.
    #[allow(clippy::only_used_in_recursion)]
    fn analyze_statements(
        &self,
        stmts: &[HirStatement],
        tree: &mut ScopeTree,
        current_scope: usize,
        base_index: usize,
    ) -> usize {
        let mut index = base_index;

        for stmt in stmts {
            match stmt {
                HirStatement::VariableDeclaration { name, .. } => {
                    // Add variable to current scope
                    tree.add_variable(current_scope, name.clone());
                    index += 1;
                }
                HirStatement::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    // Create scope for then block
                    let then_start = index + 1;
                    let then_scope =
                        tree.add_scope(current_scope, (then_start, then_start + then_block.len()));
                    index = self.analyze_statements(then_block, tree, then_scope, then_start);

                    // Create scope for else block if present
                    if let Some(else_stmts) = else_block {
                        let else_start = index;
                        let else_scope = tree
                            .add_scope(current_scope, (else_start, else_start + else_stmts.len()));
                        index = self.analyze_statements(else_stmts, tree, else_scope, else_start);
                    }
                }
                HirStatement::While { body, .. } => {
                    // Create scope for loop body
                    let body_start = index + 1;
                    let body_scope =
                        tree.add_scope(current_scope, (body_start, body_start + body.len()));
                    index = self.analyze_statements(body, tree, body_scope, body_start);
                }
                _ => {
                    index += 1;
                }
            }
        }

        index
    }

    /// Track variable lifetimes in a function.
    ///
    /// Returns a map from variable name to lifetime information.
    pub fn track_lifetimes(
        &self,
        func: &HirFunction,
        scope_tree: &ScopeTree,
    ) -> HashMap<String, VariableLifetime> {
        let mut lifetimes = HashMap::new();

        // Track each variable's lifetime
        for scope in scope_tree.scopes() {
            for var_name in &scope.variables {
                let lifetime = VariableLifetime {
                    name: var_name.clone(),
                    declared_in_scope: scope.id,
                    first_use: scope.statement_range.0,
                    last_use: scope.statement_range.1,
                    escapes: self.check_if_escapes(var_name, func),
                };
                lifetimes.insert(var_name.clone(), lifetime);
            }
        }

        lifetimes
    }

    /// Check if a variable escapes the function (returned or stored).
    fn check_if_escapes(&self, var_name: &str, func: &HirFunction) -> bool {
        // Check if variable appears in return statement
        for stmt in func.body() {
            if let HirStatement::Return(Some(expr)) = stmt {
                if self.expression_uses_variable(expr, var_name) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if an expression uses a variable.
    #[allow(clippy::only_used_in_recursion)]
    fn expression_uses_variable(&self, expr: &decy_hir::HirExpression, var_name: &str) -> bool {
        use decy_hir::HirExpression;
        match expr {
            HirExpression::Variable(name) => name == var_name,
            HirExpression::BinaryOp { left, right, .. } => {
                self.expression_uses_variable(left, var_name)
                    || self.expression_uses_variable(right, var_name)
            }
            HirExpression::Dereference(inner) | HirExpression::AddressOf(inner) => {
                self.expression_uses_variable(inner, var_name)
            }
            HirExpression::UnaryOp { operand, .. } => {
                self.expression_uses_variable(operand, var_name)
            }
            HirExpression::FunctionCall { arguments, .. } => arguments
                .iter()
                .any(|arg| self.expression_uses_variable(arg, var_name)),
            HirExpression::FieldAccess { object, .. } => {
                self.expression_uses_variable(object, var_name)
            }
            HirExpression::PointerFieldAccess { pointer, .. } => {
                self.expression_uses_variable(pointer, var_name)
            }
            HirExpression::ArrayIndex { array, index } => {
                self.expression_uses_variable(array, var_name)
                    || self.expression_uses_variable(index, var_name)
            }
            HirExpression::IntLiteral(_)
            | HirExpression::StringLiteral(_)
            | HirExpression::Sizeof { .. } => false,
        }
    }

    /// Detect potential dangling pointer issues.
    ///
    /// Returns a list of variable names that may result in dangling pointers.
    pub fn detect_dangling_pointers(
        &self,
        lifetimes: &HashMap<String, VariableLifetime>,
    ) -> Vec<String> {
        let mut dangling = Vec::new();

        for (var_name, lifetime) in lifetimes {
            // If a variable escapes but is declared in a nested scope,
            // it may create a dangling pointer
            if lifetime.escapes && lifetime.declared_in_scope > 0 {
                dangling.push(var_name.clone());
            }
        }

        dangling
    }

    /// Infer lifetime relationships between variables.
    ///
    /// Determines if one variable's lifetime outlives another.
    pub fn infer_lifetime_relationships(
        &self,
        lifetimes: &HashMap<String, VariableLifetime>,
        scope_tree: &ScopeTree,
    ) -> HashMap<(String, String), LifetimeRelation> {
        let mut relationships = HashMap::new();

        // Compare all pairs of variables
        let var_names: Vec<&String> = lifetimes.keys().collect();
        for i in 0..var_names.len() {
            for j in (i + 1)..var_names.len() {
                let var1 = var_names[i];
                let var2 = var_names[j];

                let lifetime1 = &lifetimes[var1];
                let lifetime2 = &lifetimes[var2];

                let relation = self.compare_lifetimes(lifetime1, lifetime2, scope_tree);
                relationships.insert((var1.clone(), var2.clone()), relation);
            }
        }

        relationships
    }

    /// Compare two lifetimes to determine their relationship.
    fn compare_lifetimes(
        &self,
        lifetime1: &VariableLifetime,
        lifetime2: &VariableLifetime,
        scope_tree: &ScopeTree,
    ) -> LifetimeRelation {
        // Check scope nesting
        let scope1_in_scope2 =
            scope_tree.is_nested_in(lifetime1.declared_in_scope, lifetime2.declared_in_scope);
        let scope2_in_scope1 =
            scope_tree.is_nested_in(lifetime2.declared_in_scope, lifetime1.declared_in_scope);

        if scope1_in_scope2 {
            // Variable 2 outlives variable 1 (variable 1 is in nested scope)
            LifetimeRelation::OutlivedBy
        } else if scope2_in_scope1 {
            // Variable 1 outlives variable 2
            LifetimeRelation::Outlives
        } else if lifetime1.declared_in_scope == lifetime2.declared_in_scope {
            // Same scope - equal lifetimes
            LifetimeRelation::Equal
        } else {
            // Different branches - independent
            LifetimeRelation::Independent
        }
    }
}

impl Default for LifetimeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "lifetime_tests.rs"]
mod lifetime_tests;
