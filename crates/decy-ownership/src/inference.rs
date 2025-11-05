//! Ownership inference from pointer usage patterns.
//!
//! This module infers whether pointers represent ownership or borrowing,
//! and determines mutability (&T vs &mut T).

use crate::dataflow::DataflowGraph;
use std::collections::HashMap;

/// Ownership classification for a pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipKind {
    /// Owns the data (`Box<T>`, `Vec<T>`, or owned value)
    Owning,
    /// Immutable borrow (&T)
    ImmutableBorrow,
    /// Mutable borrow (&mut T)
    MutableBorrow,
    /// Uncertain (needs manual review)
    Unknown,
}

/// Inference result for a pointer variable.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnershipInference {
    /// Variable name
    pub variable: String,
    /// Inferred ownership kind
    pub kind: OwnershipKind,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning for the inference
    pub reason: String,
}

/// Ownership inference engine.
#[derive(Debug)]
pub struct OwnershipInferencer;

impl OwnershipInferencer {
    /// Create a new ownership inferencer.
    pub fn new() -> Self {
        Self
    }

    /// Infer ownership for all pointers in a dataflow graph.
    ///
    /// Returns a map from variable name to ownership inference.
    pub fn infer(&self, graph: &DataflowGraph) -> HashMap<String, OwnershipInference> {
        let mut inferences = HashMap::new();

        // Analyze each pointer variable in the graph
        for var_name in graph.variables() {
            let kind = self.classify_pointer(var_name, graph);
            let confidence = self.calculate_confidence(&kind, graph, var_name);
            let reason = self.generate_reasoning(var_name, &kind, graph);

            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind,
                    confidence,
                    reason,
                },
            );
        }

        inferences
    }

    /// Classify a single pointer's ownership.
    fn classify_pointer(&self, var_name: &str, graph: &DataflowGraph) -> OwnershipKind {
        use crate::dataflow::NodeKind;

        // Get nodes for this variable
        let nodes = match graph.nodes_for(var_name) {
            Some(n) => n,
            None => return OwnershipKind::Unknown,
        };

        // Analyze the first (primary) node to determine ownership
        if let Some(first_node) = nodes.first() {
            match &first_node.kind {
                NodeKind::Allocation => {
                    // malloc creates an owning pointer
                    OwnershipKind::Owning
                }
                NodeKind::Parameter => {
                    // Parameters are typically borrows
                    // Check if it's mutated to determine immutable vs mutable
                    if self.is_mutated(var_name, graph) {
                        OwnershipKind::MutableBorrow
                    } else {
                        OwnershipKind::ImmutableBorrow
                    }
                }
                NodeKind::Assignment { source: _ } => {
                    // Pointer assigned from another pointer is a borrow
                    // Determine mutability based on usage
                    if self.is_mutated(var_name, graph) {
                        OwnershipKind::MutableBorrow
                    } else {
                        OwnershipKind::ImmutableBorrow
                    }
                }
                NodeKind::Dereference => {
                    // Dereference creates a borrow
                    OwnershipKind::ImmutableBorrow
                }
                NodeKind::Free => {
                    // Free indicates the pointer was owning
                    OwnershipKind::Owning
                }
                NodeKind::ArrayAllocation {
                    size: _,
                    element_type: _,
                } => {
                    // DECY-067 RED: Array allocations create owning pointers
                    // In the future (DECY-068) this will be ArrayPointer classification
                    OwnershipKind::Owning
                }
            }
        } else {
            OwnershipKind::Unknown
        }
    }

    /// Detect if a pointer is mutated.
    ///
    /// Checks if the pointer is used in dereference assignments or passed to functions
    /// that might mutate it.
    fn is_mutated(&self, var_name: &str, graph: &DataflowGraph) -> bool {
        use crate::dataflow::NodeKind;

        // Get all nodes for this variable
        let nodes = match graph.nodes_for(var_name) {
            Some(n) => n,
            None => return false,
        };

        // Check if any node indicates mutation
        for node in nodes {
            if matches!(&node.kind, NodeKind::Dereference) {
                // Dereference on LHS of assignment indicates mutation
                // For now, conservatively assume dereference might mutate
                return true;
            }
        }

        false
    }

    /// Check if a pointer escapes the function (returned or stored).
    ///
    /// A pointer escapes if it's returned from the function or assigned to
    /// a location that outlives the function.
    fn escapes_function(&self, var_name: &str, graph: &DataflowGraph) -> bool {
        use crate::dataflow::NodeKind;

        let nodes = match graph.nodes_for(var_name) {
            Some(n) => n,
            None => return false,
        };

        // Check if variable appears in any return statement
        // This is a simplified check - in reality we'd need to track
        // the actual return value
        for node in nodes {
            if matches!(node.kind, NodeKind::Assignment { .. }) {
                // Conservative: if assigned somewhere, might escape
                // Future: track assignment targets more precisely
                return true;
            }
        }

        false
    }

    /// Calculate confidence score for an inference.
    fn calculate_confidence(
        &self,
        kind: &OwnershipKind,
        graph: &DataflowGraph,
        var_name: &str,
    ) -> f32 {
        let base_confidence = match kind {
            OwnershipKind::Owning => 0.9,          // High confidence for malloc
            OwnershipKind::ImmutableBorrow => 0.8, // Medium-high for immutable
            OwnershipKind::MutableBorrow => 0.75,  // Slightly lower for mutable
            OwnershipKind::Unknown => 0.3,         // Low for uncertain
        };

        // Adjust confidence based on additional signals
        let mut confidence = base_confidence;

        // Boost confidence if pointer is returned (transfers ownership)
        if self.escapes_function(var_name, graph) {
            match kind {
                OwnershipKind::Owning => {
                    confidence = f32::min(confidence + 0.05, 0.95); // Boost owning
                }
                _ => {
                    confidence = f32::max(confidence - 0.05, 0.3); // Lower borrow confidence if escapes
                }
            }
        }

        confidence
    }

    /// Generate human-readable reasoning for an inference.
    fn generate_reasoning(
        &self,
        var_name: &str,
        kind: &OwnershipKind,
        graph: &DataflowGraph,
    ) -> String {
        use crate::dataflow::NodeKind;

        let nodes = match graph.nodes_for(var_name) {
            Some(n) => n,
            None => return format!("No information available for {}", var_name),
        };

        if let Some(first_node) = nodes.first() {
            match (&first_node.kind, kind) {
                (NodeKind::Allocation, OwnershipKind::Owning) => {
                    format!("{} allocated via malloc, owns the data", var_name)
                }
                (NodeKind::Parameter, OwnershipKind::ImmutableBorrow) => {
                    format!("{} is a parameter, used read-only", var_name)
                }
                (NodeKind::Parameter, OwnershipKind::MutableBorrow) => {
                    format!("{} is a parameter, may be modified", var_name)
                }
                (NodeKind::Assignment { source }, OwnershipKind::ImmutableBorrow) => {
                    format!("{} assigned from {}, immutable borrow", var_name, source)
                }
                (NodeKind::Assignment { source }, OwnershipKind::MutableBorrow) => {
                    format!("{} assigned from {}, mutable borrow", var_name, source)
                }
                _ => format!("{} has {:?} ownership", var_name, kind),
            }
        } else {
            format!("{} has no tracked nodes", var_name)
        }
    }
}

impl Default for OwnershipInferencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "inference_tests.rs"]
mod inference_tests;
