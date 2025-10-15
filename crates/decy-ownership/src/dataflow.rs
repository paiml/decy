//! Dataflow analysis for tracking pointer usage patterns.
//!
//! This module builds a dataflow graph that tracks how pointers flow through
//! functions, enabling detection of ownership patterns and use-after-free issues.

use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};
use std::collections::{HashMap, HashSet};

/// Represents a node in the dataflow graph (a pointer variable or operation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerNode {
    /// Variable name
    pub name: String,
    /// Statement index where this node is defined
    pub def_index: usize,
    /// Node kind (allocation, assignment, dereference, etc.)
    pub kind: NodeKind,
}

/// Kind of pointer operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Allocation (malloc, etc.)
    Allocation,
    /// Assignment from another variable
    Assignment {
        /// Source variable name
        source: String,
    },
    /// Dereference operation
    Dereference,
    /// Parameter (function parameter)
    Parameter,
    /// Free operation
    Free,
}

/// Dataflow graph tracking pointer dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataflowGraph {
    /// Map from variable name to its nodes
    nodes: HashMap<String, Vec<PointerNode>>,
    /// Dependencies: variable -> variables it depends on
    dependencies: HashMap<String, HashSet<String>>,
    /// Uses after free: variable -> indices where used after freed
    use_after_free: HashMap<String, Vec<usize>>,
}

impl DataflowGraph {
    /// Create a new empty dataflow graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependencies: HashMap::new(),
            use_after_free: HashMap::new(),
        }
    }

    /// Get all nodes for a variable.
    pub fn nodes_for(&self, var: &str) -> Option<&Vec<PointerNode>> {
        self.nodes.get(var)
    }

    /// Get dependencies for a variable.
    pub fn dependencies_for(&self, var: &str) -> Option<&HashSet<String>> {
        self.dependencies.get(var)
    }

    /// Check if a variable has use-after-free issues.
    pub fn has_use_after_free(&self, var: &str) -> bool {
        self.use_after_free.contains_key(var)
    }

    /// Get use-after-free indices for a variable.
    pub fn use_after_free_indices(&self, var: &str) -> Option<&Vec<usize>> {
        self.use_after_free.get(var)
    }

    /// Get all variables in the graph.
    pub fn variables(&self) -> Vec<&String> {
        self.nodes.keys().collect()
    }
}

impl Default for DataflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer that builds dataflow graphs from HIR functions.
#[derive(Debug)]
pub struct DataflowAnalyzer;

impl DataflowAnalyzer {
    /// Create a new dataflow analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Build a dataflow graph for a function.
    ///
    /// Analyzes pointer usage patterns throughout the function body.
    pub fn analyze(&self, func: &HirFunction) -> DataflowGraph {
        let mut graph = DataflowGraph::new();

        // Track function parameters (pointer parameters only)
        for param in func.parameters() {
            if matches!(param.param_type(), HirType::Pointer(_) | HirType::Box(_)) {
                let node = PointerNode {
                    name: param.name().to_string(),
                    def_index: 0,
                    kind: NodeKind::Parameter,
                };
                graph
                    .nodes
                    .entry(param.name().to_string())
                    .or_default()
                    .push(node);
            }
        }

        // Analyze function body
        for (index, stmt) in func.body().iter().enumerate() {
            self.track_statement(stmt, &mut graph, index);
        }

        // Detect use-after-free patterns (future enhancement)
        self.detect_use_after_free(&mut graph);

        graph
    }

    /// Track pointer operations in a statement.
    fn track_statement(&self, stmt: &HirStatement, graph: &mut DataflowGraph, index: usize) {
        match stmt {
            HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } => {
                // Only track pointer types
                if matches!(var_type, HirType::Pointer(_) | HirType::Box(_)) {
                    if let Some(init_expr) = initializer {
                        let kind = self.classify_initialization(init_expr);

                        // Add dependencies if assignment from variable
                        if let NodeKind::Assignment { ref source } = kind {
                            graph
                                .dependencies
                                .entry(name.clone())
                                .or_default()
                                .insert(source.clone());
                        }

                        let node = PointerNode {
                            name: name.clone(),
                            def_index: index,
                            kind,
                        };
                        graph.nodes.entry(name.clone()).or_default().push(node);
                    }
                }
            }
            HirStatement::Assignment { target: _, value } => {
                // Track uses of pointers in the value expression
                self.track_expression_uses(value, graph, index);
            }
            HirStatement::DerefAssignment { target, value } => {
                // Track uses of pointers in both target and value expressions
                self.track_expression_uses(target, graph, index);
                self.track_expression_uses(value, graph, index);
            }
            HirStatement::ArrayIndexAssignment {
                array,
                index: idx,
                value,
            } => {
                // Track uses of pointers in array, index, and value expressions
                self.track_expression_uses(array, graph, index);
                self.track_expression_uses(idx, graph, index);
                self.track_expression_uses(value, graph, index);
            }
            HirStatement::FieldAssignment {
                object,
                field: _,
                value,
            } => {
                // Track uses of pointers in object and value expressions
                self.track_expression_uses(object, graph, index);
                self.track_expression_uses(value, graph, index);
            }
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.track_expression_uses(condition, graph, index);
                for s in then_block {
                    self.track_statement(s, graph, index);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.track_statement(s, graph, index);
                    }
                }
            }
            HirStatement::While { condition, body } => {
                self.track_expression_uses(condition, graph, index);
                for s in body {
                    self.track_statement(s, graph, index);
                }
            }
            HirStatement::For {
                init,
                condition,
                increment,
                body,
            } => {
                // Track init statement
                if let Some(init_stmt) = init {
                    self.track_statement(init_stmt, graph, index);
                }
                // Track condition expression
                self.track_expression_uses(condition, graph, index);
                // Track loop body
                for s in body {
                    self.track_statement(s, graph, index);
                }
                // Track increment statement
                if let Some(inc_stmt) = increment {
                    self.track_statement(inc_stmt, graph, index);
                }
            }
            HirStatement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.track_expression_uses(expr, graph, index);
                }
            }
            HirStatement::Break | HirStatement::Continue => {
                // No pointer tracking needed
            }
            HirStatement::Switch {
                condition,
                cases,
                default_case,
            } => {
                self.track_expression_uses(condition, graph, index);
                for case in cases {
                    for stmt in &case.body {
                        self.track_statement(stmt, graph, index);
                    }
                }
                if let Some(default_stmts) = default_case {
                    for stmt in default_stmts {
                        self.track_statement(stmt, graph, index);
                    }
                }
            }
            HirStatement::Free { pointer } => {
                // Track free() call - marks the pointer as freed
                // Future: detect use-after-free by tracking subsequent uses
                self.track_expression_uses(pointer, graph, index);
            }
        }
    }

    /// Classify the initialization expression to determine node kind.
    fn classify_initialization(&self, expr: &HirExpression) -> NodeKind {
        match expr {
            HirExpression::FunctionCall { function, .. } if function == "malloc" => {
                NodeKind::Allocation
            }
            HirExpression::Malloc { .. } => NodeKind::Allocation,
            HirExpression::Variable(var_name) => NodeKind::Assignment {
                source: var_name.clone(),
            },
            HirExpression::Dereference(_) => NodeKind::Dereference,
            _ => NodeKind::Assignment {
                source: "unknown".to_string(),
            },
        }
    }

    /// Track uses of pointer variables in an expression.
    fn track_expression_uses(&self, expr: &HirExpression, graph: &mut DataflowGraph, index: usize) {
        Self::track_expr_recursive(expr, graph, index);
    }

    /// Recursively track expression uses (helper for track_expression_uses).
    fn track_expr_recursive(expr: &HirExpression, _graph: &mut DataflowGraph, _index: usize) {
        match expr {
            HirExpression::Variable(_) => {
                // Pointer variable used - track it
                // Future: detect use-after-free here
            }
            HirExpression::Dereference(inner) => {
                Self::track_expr_recursive(inner, _graph, _index);
            }
            HirExpression::UnaryOp { operand, .. } => {
                Self::track_expr_recursive(operand, _graph, _index);
            }
            HirExpression::BinaryOp { left, right, .. } => {
                Self::track_expr_recursive(left, _graph, _index);
                Self::track_expr_recursive(right, _graph, _index);
            }
            HirExpression::AddressOf(inner) => {
                Self::track_expr_recursive(inner, _graph, _index);
            }
            HirExpression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    Self::track_expr_recursive(arg, _graph, _index);
                }
            }
            HirExpression::FieldAccess { object, .. } => {
                Self::track_expr_recursive(object, _graph, _index);
            }
            HirExpression::PointerFieldAccess { pointer, .. } => {
                Self::track_expr_recursive(pointer, _graph, _index);
            }
            HirExpression::ArrayIndex { array, index } => {
                Self::track_expr_recursive(array, _graph, _index);
                Self::track_expr_recursive(index, _graph, _index);
            }
            HirExpression::IntLiteral(_)
            | HirExpression::StringLiteral(_)
            | HirExpression::Sizeof { .. }
            | HirExpression::NullLiteral => {
                // No tracking needed for literals, sizeof, or NULL
            }
            HirExpression::IsNotNull(inner) => {
                Self::track_expr_recursive(inner, _graph, _index);
            }
            HirExpression::Calloc { count, .. } => {
                // Track the count expression (may use variables)
                Self::track_expr_recursive(count, _graph, _index);
            }
            HirExpression::Malloc { size } => {
                // Track the size expression (may use variables)
                Self::track_expr_recursive(size, _graph, _index);
            }
            HirExpression::Realloc { pointer, new_size } => {
                // Track both pointer and new_size expressions
                Self::track_expr_recursive(pointer, _graph, _index);
                Self::track_expr_recursive(new_size, _graph, _index);
            }
        }
    }

    /// Detect use-after-free patterns.
    ///
    /// Currently a placeholder - requires ExpressionStatement support in HIR
    /// to properly detect free() calls.
    fn detect_use_after_free(&self, _graph: &mut DataflowGraph) {
        // Placeholder for future implementation
        // Will require tracking free() calls and subsequent uses
    }
}

impl Default for DataflowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "dataflow_tests.rs"]
mod dataflow_tests;
