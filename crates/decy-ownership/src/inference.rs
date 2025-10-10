//! Ownership inference from pointer usage patterns.
//!
//! This module infers whether pointers represent ownership or borrowing,
//! and determines mutability (&T vs &mut T).

use crate::dataflow::DataflowGraph;
use std::collections::HashMap;

/// Ownership classification for a pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipKind {
    /// Owns the data (Box<T>, Vec<T>, or owned value)
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
    fn classify_pointer(
        &self,
        var_name: &str,
        graph: &DataflowGraph,
    ) -> OwnershipKind {
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
            }
        } else {
            OwnershipKind::Unknown
        }
    }

    /// Detect if a pointer is mutated.
    ///
    /// Currently simplified - assumes parameters written to are mutable.
    /// Future enhancement: track actual writes through pointer dereferences.
    fn is_mutated(&self, _var_name: &str, _graph: &DataflowGraph) -> bool {
        // Simplified: assume not mutated unless we can prove otherwise
        // Future: analyze if pointer is used in assignment target position
        false
    }

    /// Check if a pointer escapes the function (returned or stored).
    ///
    /// Future enhancement for more sophisticated ownership analysis.
    fn _escapes_function(&self, _var_name: &str, _graph: &DataflowGraph) -> bool {
        // Placeholder for future implementation
        false
    }

    /// Calculate confidence score for an inference.
    fn calculate_confidence(&self, kind: &OwnershipKind, _graph: &DataflowGraph, _var_name: &str) -> f32 {
        match kind {
            OwnershipKind::Owning => {
                // High confidence for malloc allocations
                0.9
            }
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow => {
                // Medium-high confidence for borrows
                0.8
            }
            OwnershipKind::Unknown => {
                // Low confidence for uncertain cases
                0.3
            }
        }
    }

    /// Generate human-readable reasoning for an inference.
    fn generate_reasoning(&self, var_name: &str, kind: &OwnershipKind, graph: &DataflowGraph) -> String {
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
