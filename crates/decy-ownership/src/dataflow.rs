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
    /// Array allocation (stack or heap arrays)
    /// DECY-067: Added for array detection in ownership inference
    ArrayAllocation {
        /// Array size (None for runtime-sized arrays)
        size: Option<usize>,
        /// Element type
        element_type: HirType,
    },
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
    /// DECY-067 GREEN: Array base tracking (pointer -> array it's derived from)
    array_bases: HashMap<String, String>,
    /// DECY-071 GREEN: Function parameters for array detection
    parameters: Vec<decy_hir::HirParameter>,
    /// DECY-071 GREEN: Function body for usage analysis
    body: Vec<decy_hir::HirStatement>,
}

impl DataflowGraph {
    /// Create a new empty dataflow graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependencies: HashMap::new(),
            use_after_free: HashMap::new(),
            array_bases: HashMap::new(), // DECY-067 GREEN
            parameters: Vec::new(),      // DECY-071 GREEN
            body: Vec::new(),            // DECY-071 GREEN
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

    /// Get the array base for a pointer variable (if it's derived from an array).
    /// DECY-067 GREEN: Real implementation
    pub fn array_base_for(&self, var: &str) -> Option<&str> {
        self.array_bases.get(var).map(|s| s.as_str())
    }

    /// Check if a parameter is an array pointer (has associated length parameter).
    /// DECY-071 GREEN: Proper implementation with multiple heuristics
    /// Detects the pattern: fn(int* arr, int len) where pointer param followed by int param
    pub fn is_array_parameter(&self, var: &str) -> Option<bool> {
        // Find the parameter in the parameter list
        let param_index = self.parameters.iter().position(|p| p.name() == var)?;
        let param = &self.parameters[param_index];

        // Only check pointer parameters
        if !matches!(param.param_type(), HirType::Pointer(_)) {
            return Some(false);
        }

        // Conservative: Don't detect struct pointers as arrays
        // Struct arrays are ambiguous without more context
        if let HirType::Pointer(inner) = param.param_type() {
            if matches!(**inner, HirType::Struct(_)) {
                return Some(false);
            }
        }

        let mut confidence = 0;
        let mut signals = 0;

        // Heuristic 1: Check if followed by an integer parameter (length param)
        // Pattern: (T* arr, int len) or (T* arr, size_t size)
        if param_index + 1 < self.parameters.len() {
            let next_param = &self.parameters[param_index + 1];
            if matches!(next_param.param_type(), HirType::Int) {
                confidence += 3; // Strong signal
                signals += 1;
            }
        }

        // Heuristic 2: Check parameter naming patterns
        // Common array names: arr, array, buf, buffer, data, items
        // Common length names: len, length, size, count, num
        let param_name = param.name().to_lowercase();
        if param_name.contains("arr")
            || param_name.contains("buf")
            || param_name == "data"
            || param_name == "items"
        {
            confidence += 2; // Moderate signal
            signals += 1;
        }

        // Check if next param has length-like name
        if param_index + 1 < self.parameters.len() {
            let next_name = self.parameters[param_index + 1].name().to_lowercase();
            if next_name.contains("len")
                || next_name.contains("size")
                || next_name.contains("count")
                || next_name.contains("num")
            {
                confidence += 2; // Moderate signal
                signals += 1;
            }
        }

        // Heuristic 3: Check for array indexing usage in function body
        if self.has_array_indexing(var) {
            confidence += 3; // Strong signal
            signals += 1;
        }

        // Heuristic 4: Check for pointer arithmetic (negative signal)
        if self.has_pointer_arithmetic(var) {
            confidence -= 2; // Pointer arithmetic suggests non-array usage
            signals += 1;
        }

        // Decision: require at least 2 signals and positive confidence
        if signals >= 2 && confidence >= 3 {
            Some(true)
        } else {
            Some(false)
        }
    }

    /// Get all array parameters and their corresponding length parameters.
    /// DECY-072 GREEN: Returns (array_param_name, length_param_name) pairs
    ///
    /// # Returns
    ///
    /// A vector of tuples where:
    /// - First element is the array parameter name
    /// - Second element is the corresponding length parameter name (if any)
    ///
    /// # Examples
    ///
    /// For `fn process(int* arr, int len)` returns `[("arr", Some("len"))]`
    /// For `fn process(int* ptr)` returns `[]` (not an array parameter)
    pub fn get_array_parameters(&self) -> Vec<(String, Option<String>)> {
        let mut array_params = Vec::new();

        for (i, param) in self.parameters.iter().enumerate() {
            // Only check pointer parameters
            if !matches!(param.param_type(), HirType::Pointer(_)) {
                continue;
            }

            // Check if this is an array parameter
            if let Some(true) = self.is_array_parameter(param.name()) {
                // Try to find the corresponding length parameter
                let length_param = if i + 1 < self.parameters.len() {
                    let next_param = &self.parameters[i + 1];
                    // Check if next parameter is integer type (likely a length)
                    if matches!(next_param.param_type(), HirType::Int) {
                        Some(next_param.name().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                array_params.push((param.name().to_string(), length_param));
            }
        }

        array_params
    }

    /// Check if a variable is used with array indexing in the function body.
    /// DECY-071 GREEN: Helper for array detection
    fn has_array_indexing(&self, var: &str) -> bool {
        for stmt in &self.body {
            if self.statement_has_array_indexing(stmt, var) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement contains array indexing for a variable.
    #[allow(clippy::only_used_in_recursion)]
    fn statement_has_array_indexing(&self, stmt: &HirStatement, var: &str) -> bool {
        match stmt {
            HirStatement::ArrayIndexAssignment { array, .. } => {
                if let HirExpression::Variable(name) = &**array {
                    return name == var;
                }
                false
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_has_array_indexing(s, var))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_has_array_indexing(s, var))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_has_array_indexing(s, var)),
            _ => false,
        }
    }

    /// Check if a variable is used with pointer arithmetic in the function body.
    /// DECY-071 GREEN: Helper for array detection (negative signal)
    fn has_pointer_arithmetic(&self, var: &str) -> bool {
        for stmt in &self.body {
            if self.statement_has_pointer_arithmetic(stmt, var) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement contains pointer arithmetic for a variable.
    fn statement_has_pointer_arithmetic(&self, stmt: &HirStatement, var: &str) -> bool {
        match stmt {
            HirStatement::VariableDeclaration { initializer, .. } => {
                if let Some(expr) = initializer {
                    return self.expression_has_pointer_arithmetic(expr, var);
                }
                false
            }
            HirStatement::Assignment { value, .. } => {
                self.expression_has_pointer_arithmetic(value, var)
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                then_block
                    .iter()
                    .any(|s| self.statement_has_pointer_arithmetic(s, var))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter()
                            .any(|s| self.statement_has_pointer_arithmetic(s, var))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
                .iter()
                .any(|s| self.statement_has_pointer_arithmetic(s, var)),
            _ => false,
        }
    }

    /// Check if an expression contains pointer arithmetic for a variable.
    fn expression_has_pointer_arithmetic(&self, expr: &decy_hir::HirExpression, var: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right: _ } => {
                // Check if this is pointer arithmetic (ptr + offset or ptr - offset)
                if matches!(
                    op,
                    decy_hir::BinaryOperator::Add | decy_hir::BinaryOperator::Subtract
                ) {
                    if let HirExpression::Variable(name) = &**left {
                        return name == var;
                    }
                }
                false
            }
            _ => false,
        }
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

        // DECY-071 GREEN: Store parameters and body for array detection
        graph.parameters = func.parameters().to_vec();
        graph.body = func.body().to_vec();

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
                // DECY-067 GREEN: Detect array allocations (stack arrays)
                if let HirType::Array { element_type, size } = var_type {
                    // Stack array allocation: int arr[10];
                    let node = PointerNode {
                        name: name.clone(),
                        def_index: index,
                        kind: NodeKind::ArrayAllocation {
                            size: *size,
                            element_type: (**element_type).clone(),
                        },
                    };
                    graph.nodes.entry(name.clone()).or_default().push(node);
                    return; // Early return after handling array
                }

                // Track pointer types
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

                            // DECY-067 GREEN: Track if assigned from an array
                            // Check if source is an array by looking up its node
                            if let Some(source_nodes) = graph.nodes.get(source) {
                                if let Some(first_node) = source_nodes.first() {
                                    if matches!(first_node.kind, NodeKind::ArrayAllocation { .. }) {
                                        // This pointer is derived from an array!
                                        graph.array_bases.insert(name.clone(), source.clone());
                                    }
                                }
                            }
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
            HirStatement::Expression(expr) => {
                // Track pointer uses in expression statements (DECY-065)
                // e.g., printf() calls, function calls, etc.
                self.track_expression_uses(expr, graph, index);
            }
        }
    }

    /// Classify the initialization expression to determine node kind.
    fn classify_initialization(&self, expr: &HirExpression) -> NodeKind {
        match expr {
            HirExpression::FunctionCall {
                function,
                arguments,
            } if function == "malloc" => {
                // DECY-067 GREEN: Detect heap array pattern: malloc(n * sizeof(T))
                // Clippy: Collapse nested if-let into single pattern
                if let Some(HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    right,
                    ..
                }) = arguments.first()
                {
                    // Check if multiplying by sizeof
                    if matches!(**right, HirExpression::Sizeof { .. }) {
                        // This is an array allocation pattern
                        // Extract element type from sizeof
                        if let HirExpression::Sizeof { type_name } = &**right {
                            // Map type name to HirType (simplified)
                            let element_type = match type_name.as_str() {
                                "int" => HirType::Int,
                                "char" => HirType::Char,
                                "float" => HirType::Float,
                                "double" => HirType::Double,
                                _ => HirType::Int, // Default fallback
                            };
                            return NodeKind::ArrayAllocation {
                                size: None, // Runtime size
                                element_type,
                            };
                        }
                    }
                }
                // Regular malloc (not array pattern)
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
            HirExpression::Cast { expr, .. } => {
                // Track the expression being cast
                Self::track_expr_recursive(expr, _graph, _index);
            }
            HirExpression::CompoundLiteral { initializers, .. } => {
                // Track all initializer expressions
                for init in initializers {
                    Self::track_expr_recursive(init, _graph, _index);
                }
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
            HirExpression::StringMethodCall {
                receiver,
                arguments,
                ..
            } => {
                // Track receiver and arguments
                Self::track_expr_recursive(receiver, _graph, _index);
                for arg in arguments {
                    Self::track_expr_recursive(arg, _graph, _index);
                }
            }
            HirExpression::SliceIndex { slice, index, .. } => {
                // DECY-069: Track safe slice indexing
                Self::track_expr_recursive(slice, _graph, _index);
                Self::track_expr_recursive(index, _graph, _index);
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
