//! ML-enhanced ownership inference features.
//!
//! DECY-ML-001: OwnershipFeatures struct for ML-based classification
//! DECY-ML-003: Ownership defect taxonomy (8 categories)
//!
//! Based on:
//! - Type4Py (ICSE 2022): Similarity learning for type inference
//! - Typilus (PLDI 2020): GNN-based type hints with data flow
//! - OIP: Hybrid classification with confidence fallback

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// DECY-ML-003: OWNERSHIP DEFECT TAXONOMY
// ============================================================================

/// Severity level for ownership defects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - optimization opportunity
    Info,
    /// Medium - suboptimal but safe
    Medium,
    /// High - incorrect but may compile
    High,
    /// Critical - causes memory unsafety
    Critical,
}

/// Ownership inference defect categories.
///
/// Based on OIP's 18-category system, focused on 8 transpiler-specific
/// categories for C→Rust ownership inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnershipDefect {
    /// DECY-O-001: Owning pointer classified as borrowing or vice versa
    PointerMisclassification,
    /// DECY-O-002: Missing or incorrect lifetime annotations
    LifetimeInferenceGap,
    /// DECY-O-003: Use-after-free pattern not caught
    DanglingPointerRisk,
    /// DECY-O-004: Multiple mutable aliases generated
    AliasViolation,
    /// DECY-O-005: Unnecessary unsafe blocks in output
    UnsafeMinimizationFailure,
    /// DECY-O-006: Array vs slice semantics error
    ArraySliceMismatch,
    /// DECY-O-007: Allocation without corresponding deallocation
    ResourceLeakPattern,
    /// DECY-O-008: Const pointer vs mutable reference error
    MutabilityMismatch,
}

impl OwnershipDefect {
    /// Get the defect code (DECY-O-XXX format).
    pub fn code(&self) -> &'static str {
        match self {
            Self::PointerMisclassification => "DECY-O-001",
            Self::LifetimeInferenceGap => "DECY-O-002",
            Self::DanglingPointerRisk => "DECY-O-003",
            Self::AliasViolation => "DECY-O-004",
            Self::UnsafeMinimizationFailure => "DECY-O-005",
            Self::ArraySliceMismatch => "DECY-O-006",
            Self::ResourceLeakPattern => "DECY-O-007",
            Self::MutabilityMismatch => "DECY-O-008",
        }
    }

    /// Get human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::PointerMisclassification => {
                "Owning pointer classified as borrowing or vice versa"
            }
            Self::LifetimeInferenceGap => "Missing or incorrect lifetime annotations",
            Self::DanglingPointerRisk => "Use-after-free pattern not caught",
            Self::AliasViolation => "Multiple mutable aliases generated",
            Self::UnsafeMinimizationFailure => "Unnecessary unsafe blocks in output",
            Self::ArraySliceMismatch => "Array vs slice semantics error",
            Self::ResourceLeakPattern => "Allocation without corresponding deallocation",
            Self::MutabilityMismatch => "Const pointer vs mutable reference error",
        }
    }

    /// Get severity level for this defect.
    pub fn severity(&self) -> Severity {
        match self {
            // Critical: memory safety violations
            Self::DanglingPointerRisk | Self::AliasViolation => Severity::Critical,
            // High: incorrect but may compile
            Self::PointerMisclassification
            | Self::LifetimeInferenceGap
            | Self::MutabilityMismatch => Severity::High,
            // Medium: suboptimal but safe
            Self::UnsafeMinimizationFailure
            | Self::ArraySliceMismatch
            | Self::ResourceLeakPattern => Severity::Medium,
        }
    }

    /// Parse defect from code string.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "DECY-O-001" => Some(Self::PointerMisclassification),
            "DECY-O-002" => Some(Self::LifetimeInferenceGap),
            "DECY-O-003" => Some(Self::DanglingPointerRisk),
            "DECY-O-004" => Some(Self::AliasViolation),
            "DECY-O-005" => Some(Self::UnsafeMinimizationFailure),
            "DECY-O-006" => Some(Self::ArraySliceMismatch),
            "DECY-O-007" => Some(Self::ResourceLeakPattern),
            "DECY-O-008" => Some(Self::MutabilityMismatch),
            _ => None,
        }
    }
}

impl fmt::Display for OwnershipDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code(), self.description())
    }
}

// ============================================================================
// DECY-ML-001: OWNERSHIP FEATURES STRUCT
// ============================================================================

/// Kind of memory allocation site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AllocationKind {
    /// malloc() call
    Malloc,
    /// calloc() call
    Calloc,
    /// realloc() call
    Realloc,
    /// Stack allocation (local variable)
    Stack,
    /// Static/global allocation
    Static,
    /// Function parameter (externally provided)
    Parameter,
    /// Unknown allocation source
    #[default]
    Unknown,
}

/// Features for ML-based ownership classification.
///
/// 142-dimension feature vector for batch processing, following:
/// - Type4Py: Similarity learning approach
/// - Typilus: Data flow integration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnershipFeatures {
    // Syntactic features (4)
    /// Pointer indirection depth (int*, int**, etc.)
    pub pointer_depth: u8,
    /// const qualifier present
    pub is_const: bool,
    /// Array decay pattern (T[] → T* parameter)
    pub is_array_decay: bool,
    /// Has accompanying size parameter (T* arr, size_t n)
    pub has_size_param: bool,

    // Semantic features from dataflow (4)
    /// How the memory was allocated
    pub allocation_site: AllocationKind,
    /// Number of free() calls on this pointer
    pub deallocation_count: u8,
    /// Number of aliases to this pointer
    pub alias_count: u8,
    /// Whether pointer escapes function scope
    pub escape_scope: bool,

    // Usage pattern features (4)
    /// Dereference for read count
    pub read_count: u32,
    /// Dereference for write count
    pub write_count: u32,
    /// Pointer arithmetic operations (p++, p+n)
    pub arithmetic_ops: u8,
    /// Null check patterns (if (p != NULL))
    pub null_checks: u8,
}

impl OwnershipFeatures {
    /// Fixed dimension for batch ML processing.
    /// Syntactic(4) + Semantic(4) + Usage(4) = 12 core features
    /// + padding for embeddings = 142 total
    pub const DIMENSION: usize = 142;

    /// Create a new builder for OwnershipFeatures.
    pub fn builder() -> OwnershipFeaturesBuilder {
        OwnershipFeaturesBuilder::default()
    }

    /// Convert features to a flat f32 vector for ML input.
    pub fn to_vector(&self) -> Vec<f32> {
        let mut vec = Vec::with_capacity(Self::DIMENSION);

        // Syntactic features
        vec.push(self.pointer_depth as f32);
        vec.push(if self.is_const { 1.0 } else { 0.0 });
        vec.push(if self.is_array_decay { 1.0 } else { 0.0 });
        vec.push(if self.has_size_param { 1.0 } else { 0.0 });

        // Semantic features
        vec.push(self.allocation_kind_to_f32());
        vec.push(self.deallocation_count as f32);
        vec.push(self.alias_count as f32);
        vec.push(if self.escape_scope { 1.0 } else { 0.0 });

        // Usage patterns
        vec.push(self.read_count as f32);
        vec.push(self.write_count as f32);
        vec.push(self.arithmetic_ops as f32);
        vec.push(self.null_checks as f32);

        // Pad to DIMENSION (reserved for embeddings in future)
        vec.resize(Self::DIMENSION, 0.0);

        vec
    }

    fn allocation_kind_to_f32(&self) -> f32 {
        match self.allocation_site {
            AllocationKind::Malloc => 1.0,
            AllocationKind::Calloc => 2.0,
            AllocationKind::Realloc => 3.0,
            AllocationKind::Stack => 4.0,
            AllocationKind::Static => 5.0,
            AllocationKind::Parameter => 6.0,
            AllocationKind::Unknown => 0.0,
        }
    }
}

/// Builder for OwnershipFeatures.
#[derive(Debug, Default)]
pub struct OwnershipFeaturesBuilder {
    features: OwnershipFeatures,
}

impl OwnershipFeaturesBuilder {
    /// Set pointer depth.
    pub fn pointer_depth(mut self, depth: u8) -> Self {
        self.features.pointer_depth = depth;
        self
    }

    /// Set const qualifier.
    pub fn const_qualified(mut self, is_const: bool) -> Self {
        self.features.is_const = is_const;
        self
    }

    /// Set array decay flag.
    pub fn array_decay(mut self, is_decay: bool) -> Self {
        self.features.is_array_decay = is_decay;
        self
    }

    /// Set size parameter flag.
    pub fn has_size_param(mut self, has_size: bool) -> Self {
        self.features.has_size_param = has_size;
        self
    }

    /// Set allocation site.
    pub fn allocation_site(mut self, kind: AllocationKind) -> Self {
        self.features.allocation_site = kind;
        self
    }

    /// Set deallocation count.
    pub fn deallocation_count(mut self, count: u8) -> Self {
        self.features.deallocation_count = count;
        self
    }

    /// Set alias count.
    pub fn alias_count(mut self, count: u8) -> Self {
        self.features.alias_count = count;
        self
    }

    /// Set escape scope flag.
    pub fn escape_scope(mut self, escapes: bool) -> Self {
        self.features.escape_scope = escapes;
        self
    }

    /// Set read count.
    pub fn read_count(mut self, count: u32) -> Self {
        self.features.read_count = count;
        self
    }

    /// Set write count.
    pub fn write_count(mut self, count: u32) -> Self {
        self.features.write_count = count;
        self
    }

    /// Set arithmetic operations count.
    pub fn arithmetic_ops(mut self, count: u8) -> Self {
        self.features.arithmetic_ops = count;
        self
    }

    /// Set null checks count.
    pub fn null_checks(mut self, count: u8) -> Self {
        self.features.null_checks = count;
        self
    }

    /// Build the OwnershipFeatures.
    pub fn build(self) -> OwnershipFeatures {
        self.features
    }
}

// ============================================================================
// INFERRED OWNERSHIP KIND
// ============================================================================

/// Inferred Rust ownership kind from C pointer analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InferredOwnership {
    /// Box<T> - owned heap allocation
    Owned,
    /// &T - immutable borrow
    Borrowed,
    /// &mut T - mutable borrow
    BorrowedMut,
    /// Rc<T> or Arc<T> - shared ownership
    Shared,
    /// *const T or *mut T - raw pointer (requires unsafe)
    RawPointer,
    /// Vec<T> - owned dynamic array
    Vec,
    /// &[T] - immutable slice
    Slice,
    /// &mut [T] - mutable slice
    SliceMut,
}

impl InferredOwnership {
    /// Convert to Rust type string.
    pub fn to_rust_type(self, inner_type: &str) -> String {
        match self {
            Self::Owned => format!("Box<{}>", inner_type),
            Self::Borrowed => format!("&{}", inner_type),
            Self::BorrowedMut => format!("&mut {}", inner_type),
            Self::Shared => format!("Rc<{}>", inner_type),
            Self::RawPointer => format!("*const {}", inner_type),
            Self::Vec => format!("Vec<{}>", inner_type),
            Self::Slice => format!("&[{}]", inner_type),
            Self::SliceMut => format!("&mut [{}]", inner_type),
        }
    }

    /// Whether this ownership kind requires unsafe code.
    pub fn requires_unsafe(&self) -> bool {
        matches!(self, Self::RawPointer)
    }
}

/// Ownership prediction with confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipPrediction {
    /// Predicted ownership kind
    pub kind: InferredOwnership,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Fallback if confidence is below threshold
    pub fallback: Option<InferredOwnership>,
}

impl OwnershipPrediction {
    /// Default confidence threshold (from spec: 0.65).
    pub const CONFIDENCE_THRESHOLD: f32 = 0.65;

    /// Check if prediction is confident enough to use.
    pub fn is_confident(&self) -> bool {
        self.confidence >= Self::CONFIDENCE_THRESHOLD
    }

    /// Get the ownership to use (predicted if confident, fallback otherwise).
    pub fn effective_ownership(&self) -> InferredOwnership {
        if self.is_confident() {
            self.kind
        } else {
            self.fallback.unwrap_or(InferredOwnership::RawPointer)
        }
    }
}

impl PartialEq for OwnershipPrediction {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && (self.confidence - other.confidence).abs() < f32::EPSILON
    }
}

// ============================================================================
// DECY-ML-002: FEATURE EXTRACTION FROM HIR
// ============================================================================

use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Extracts OwnershipFeatures from HIR functions.
///
/// DECY-ML-002: Converts HIR analysis into 142-dimension feature vectors
/// for ML-based ownership classification.
#[derive(Debug, Default)]
pub struct FeatureExtractor {
    /// Count of features extracted (for statistics)
    extracted_count: u64,
}

impl FeatureExtractor {
    /// Create a new feature extractor.
    pub fn new() -> Self {
        Self { extracted_count: 0 }
    }

    /// Get count of features extracted.
    pub fn extracted_count(&self) -> u64 {
        self.extracted_count
    }

    /// Extract features for a specific parameter by name.
    pub fn extract_for_parameter(
        &self,
        func: &HirFunction,
        param_name: &str,
    ) -> Option<OwnershipFeatures> {
        // Find the parameter
        let param = func.parameters().iter().find(|p| p.name() == param_name)?;
        let param_type = param.param_type();

        // Only extract features for pointer-like types
        if !self.is_pointer_like(param_type) {
            return None;
        }

        let param_index = func
            .parameters()
            .iter()
            .position(|p| p.name() == param_name)?;

        Some(self.extract_features(func, param_name, param_type, param_index, true))
    }

    /// Extract features for a local variable by name.
    pub fn extract_for_variable(
        &self,
        func: &HirFunction,
        var_name: &str,
    ) -> Option<OwnershipFeatures> {
        // Find variable declaration in body
        for stmt in func.body() {
            if let HirStatement::VariableDeclaration {
                name,
                var_type,
                initializer,
            } = stmt
            {
                if name == var_name && self.is_pointer_like(var_type) {
                    let allocation = self.classify_allocation(initializer.as_ref());
                    let mut features = self.extract_features(func, var_name, var_type, 0, false);
                    features.allocation_site = allocation;
                    return Some(features);
                }
            }
        }
        None
    }

    /// Extract features for all pointer parameters in a function.
    pub fn extract_all(&self, func: &HirFunction) -> Vec<(String, OwnershipFeatures)> {
        let mut result = Vec::new();

        for (idx, param) in func.parameters().iter().enumerate() {
            if self.is_pointer_like(param.param_type()) {
                let features =
                    self.extract_features(func, param.name(), param.param_type(), idx, true);
                result.push((param.name().to_string(), features));
            }
        }

        result
    }

    /// Check if a type is pointer-like (pointer, reference, box, etc.).
    fn is_pointer_like(&self, ty: &HirType) -> bool {
        matches!(
            ty,
            HirType::Pointer(_) | HirType::Box(_) | HirType::Reference { .. } | HirType::Vec(_)
        )
    }

    /// Extract features for a pointer variable.
    fn extract_features(
        &self,
        func: &HirFunction,
        var_name: &str,
        var_type: &HirType,
        param_index: usize,
        is_param: bool,
    ) -> OwnershipFeatures {
        let (reads, writes) = self.count_accesses(func.body(), var_name);

        OwnershipFeatures {
            // Syntactic features
            pointer_depth: self.compute_pointer_depth(var_type),
            is_const: self.is_const_type(var_type),
            is_array_decay: self.detect_array_decay(func, param_index),
            has_size_param: self.has_size_parameter(func, param_index),
            // Semantic features
            allocation_site: if is_param {
                AllocationKind::Parameter
            } else {
                AllocationKind::Unknown
            },
            deallocation_count: self.count_deallocations(func.body(), var_name),
            alias_count: self.count_aliases(func.body(), var_name),
            escape_scope: self.check_escape(func, var_name),
            // Usage patterns
            read_count: reads,
            write_count: writes,
            arithmetic_ops: self.count_pointer_arithmetic(func.body(), var_name),
            null_checks: self.count_null_checks(func.body(), var_name),
        }
    }

    /// Compute pointer indirection depth.
    fn compute_pointer_depth(&self, ty: &HirType) -> u8 {
        match ty {
            HirType::Pointer(inner) => 1 + self.compute_pointer_depth(inner),
            HirType::Box(inner) => 1 + self.compute_pointer_depth(inner),
            HirType::Reference { inner, .. } => 1 + self.compute_pointer_depth(inner),
            _ => 0,
        }
    }

    /// Check if type is const/immutable.
    fn is_const_type(&self, ty: &HirType) -> bool {
        match ty {
            HirType::Reference { mutable, .. } => !mutable,
            _ => false,
        }
    }

    /// Detect array decay pattern (pointer followed by size).
    fn detect_array_decay(&self, func: &HirFunction, param_index: usize) -> bool {
        let params = func.parameters();
        if param_index + 1 >= params.len() {
            return false;
        }

        // Check if current is pointer and next is integer
        let current = &params[param_index];
        let next = &params[param_index + 1];

        if !matches!(current.param_type(), HirType::Pointer(_)) {
            return false;
        }

        if !matches!(next.param_type(), HirType::Int | HirType::UnsignedInt) {
            return false;
        }

        // Check naming patterns
        let next_name = next.name().to_lowercase();
        next_name.contains("len")
            || next_name.contains("size")
            || next_name.contains("count")
            || next_name.contains("num")
            || next_name == "n"
    }

    /// Check if there's an associated size parameter.
    fn has_size_parameter(&self, func: &HirFunction, param_index: usize) -> bool {
        self.detect_array_decay(func, param_index)
    }

    /// Classify allocation kind from initializer.
    fn classify_allocation(&self, initializer: Option<&HirExpression>) -> AllocationKind {
        match initializer {
            Some(HirExpression::Malloc { .. }) => AllocationKind::Malloc,
            Some(HirExpression::Calloc { .. }) => AllocationKind::Calloc,
            Some(HirExpression::Realloc { .. }) => AllocationKind::Realloc,
            Some(HirExpression::FunctionCall { function, .. }) if function == "malloc" => {
                AllocationKind::Malloc
            }
            Some(HirExpression::FunctionCall { function, .. }) if function == "calloc" => {
                AllocationKind::Calloc
            }
            Some(HirExpression::FunctionCall { function, .. }) if function == "realloc" => {
                AllocationKind::Realloc
            }
            _ => AllocationKind::Unknown,
        }
    }

    /// Count free() calls on a variable.
    fn count_deallocations(&self, body: &[HirStatement], var_name: &str) -> u8 {
        let mut count = 0u8;
        for stmt in body {
            count = count.saturating_add(self.count_free_in_stmt(stmt, var_name));
        }
        count
    }

    fn count_free_in_stmt(&self, stmt: &HirStatement, var_name: &str) -> u8 {
        match stmt {
            HirStatement::Free { pointer } => {
                if self.expr_uses_var(pointer, var_name) {
                    1
                } else {
                    0
                }
            }
            HirStatement::If {
                then_block,
                else_block,
                ..
            } => {
                let mut count = 0u8;
                for s in then_block {
                    count = count.saturating_add(self.count_free_in_stmt(s, var_name));
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        count = count.saturating_add(self.count_free_in_stmt(s, var_name));
                    }
                }
                count
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                let mut count = 0u8;
                for s in body {
                    count = count.saturating_add(self.count_free_in_stmt(s, var_name));
                }
                count
            }
            _ => 0,
        }
    }

    /// Count aliases (assignments to other variables).
    fn count_aliases(&self, _body: &[HirStatement], _var_name: &str) -> u8 {
        // Simplified: would need full dataflow analysis
        0
    }

    /// Check if variable escapes function scope.
    fn check_escape(&self, func: &HirFunction, var_name: &str) -> bool {
        for stmt in func.body() {
            if let HirStatement::Return(Some(expr)) = stmt {
                if self.expr_uses_var(expr, var_name) {
                    return true;
                }
            }
        }
        false
    }

    /// Count read/write accesses to a variable.
    fn count_accesses(&self, body: &[HirStatement], var_name: &str) -> (u32, u32) {
        let mut reads = 0u32;
        let mut writes = 0u32;

        for stmt in body {
            let (r, w) = self.count_stmt_accesses(stmt, var_name);
            reads = reads.saturating_add(r);
            writes = writes.saturating_add(w);
        }

        (reads, writes)
    }

    fn count_stmt_accesses(&self, stmt: &HirStatement, var_name: &str) -> (u32, u32) {
        match stmt {
            HirStatement::Assignment { target, value } => {
                let mut reads: u32 = if self.expr_uses_var(value, var_name) {
                    1
                } else {
                    0
                };
                let writes: u32 = if target == var_name { 1 } else { 0 };
                // Target might also be a dereference read
                if target != var_name && self.expr_uses_var(value, var_name) {
                    reads += 1;
                }
                (reads, writes)
            }
            HirStatement::DerefAssignment { target, value } => {
                let reads: u32 = if self.expr_uses_var(value, var_name)
                    || self.expr_uses_var(target, var_name)
                {
                    1
                } else {
                    0
                };
                let writes: u32 = if self.expr_uses_var(target, var_name) {
                    1
                } else {
                    0
                };
                (reads, writes)
            }
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut reads: u32 = if self.expr_uses_var(condition, var_name) {
                    1
                } else {
                    0
                };
                let mut writes: u32 = 0;
                for s in then_block {
                    let (r, w) = self.count_stmt_accesses(s, var_name);
                    reads = reads.saturating_add(r);
                    writes = writes.saturating_add(w);
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        let (r, w) = self.count_stmt_accesses(s, var_name);
                        reads = reads.saturating_add(r);
                        writes = writes.saturating_add(w);
                    }
                }
                (reads, writes)
            }
            _ => (0, 0),
        }
    }

    /// Count pointer arithmetic operations.
    fn count_pointer_arithmetic(&self, _body: &[HirStatement], _var_name: &str) -> u8 {
        // Simplified: would need expression analysis
        0
    }

    /// Count null checks on a variable.
    fn count_null_checks(&self, body: &[HirStatement], var_name: &str) -> u8 {
        let mut count = 0u8;
        for stmt in body {
            count = count.saturating_add(self.count_null_checks_in_stmt(stmt, var_name));
        }
        count
    }

    fn count_null_checks_in_stmt(&self, stmt: &HirStatement, var_name: &str) -> u8 {
        match stmt {
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut count: u8 = if self.is_null_check(condition, var_name) {
                    1
                } else {
                    0
                };
                for s in then_block {
                    count = count.saturating_add(self.count_null_checks_in_stmt(s, var_name));
                }
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        count = count.saturating_add(self.count_null_checks_in_stmt(s, var_name));
                    }
                }
                count
            }
            HirStatement::While { condition, body } => {
                let mut count: u8 = if self.is_null_check(condition, var_name) {
                    1
                } else {
                    0
                };
                for s in body {
                    count = count.saturating_add(self.count_null_checks_in_stmt(s, var_name));
                }
                count
            }
            _ => 0,
        }
    }

    /// Check if expression is a null check for the variable.
    fn is_null_check(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::IsNotNull(inner) => self.expr_uses_var(inner, var_name),
            HirExpression::BinaryOp { left, right, .. } => {
                // Check for ptr != NULL or ptr == NULL patterns
                (self.expr_uses_var(left, var_name)
                    && matches!(**right, HirExpression::NullLiteral))
                    || (self.expr_uses_var(right, var_name)
                        && matches!(**left, HirExpression::NullLiteral))
            }
            _ => false,
        }
    }

    /// Check if expression uses a variable.
    fn expr_uses_var(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::Variable(name) => name == var_name,
            HirExpression::Dereference(inner) => self.expr_uses_var(inner, var_name),
            HirExpression::AddressOf(inner) => self.expr_uses_var(inner, var_name),
            HirExpression::BinaryOp { left, right, .. } => {
                self.expr_uses_var(left, var_name) || self.expr_uses_var(right, var_name)
            }
            HirExpression::UnaryOp { operand, .. } => self.expr_uses_var(operand, var_name),
            HirExpression::ArrayIndex { array, index } => {
                self.expr_uses_var(array, var_name) || self.expr_uses_var(index, var_name)
            }
            HirExpression::FunctionCall { arguments, .. } => arguments
                .iter()
                .any(|arg| self.expr_uses_var(arg, var_name)),
            HirExpression::IsNotNull(inner) => self.expr_uses_var(inner, var_name),
            _ => false,
        }
    }
}

// ============================================================================
// DECY-ML-009: ERROR PATTERN LIBRARY
// ============================================================================

/// Error kind for ownership inference failures.
///
/// Maps to OwnershipDefect for consistent categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnershipErrorKind {
    /// Owning pointer classified as borrowing or vice versa
    PointerMisclassification,
    /// Missing or incorrect lifetime annotations
    LifetimeInferenceGap,
    /// Use-after-free pattern not caught
    DanglingPointerRisk,
    /// Multiple mutable aliases generated
    AliasViolation,
    /// Unnecessary unsafe blocks in output
    UnsafeMinimizationFailure,
    /// Array vs slice semantics error
    ArraySliceMismatch,
    /// Allocation without corresponding deallocation
    ResourceLeakPattern,
    /// Const pointer vs mutable reference error
    MutabilityMismatch,
}

impl OwnershipErrorKind {
    /// Convert to OwnershipDefect for taxonomy integration.
    pub fn to_defect(self) -> OwnershipDefect {
        match self {
            Self::PointerMisclassification => OwnershipDefect::PointerMisclassification,
            Self::LifetimeInferenceGap => OwnershipDefect::LifetimeInferenceGap,
            Self::DanglingPointerRisk => OwnershipDefect::DanglingPointerRisk,
            Self::AliasViolation => OwnershipDefect::AliasViolation,
            Self::UnsafeMinimizationFailure => OwnershipDefect::UnsafeMinimizationFailure,
            Self::ArraySliceMismatch => OwnershipDefect::ArraySliceMismatch,
            Self::ResourceLeakPattern => OwnershipDefect::ResourceLeakPattern,
            Self::MutabilityMismatch => OwnershipDefect::MutabilityMismatch,
        }
    }
}

/// Severity level for error patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ErrorSeverity {
    /// Informational - optimization opportunity
    #[default]
    Info,
    /// Warning - suboptimal but compiles
    Warning,
    /// Error - does not compile
    Error,
    /// Critical - memory safety issue
    Critical,
}

/// Suggested fix for an error pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    /// Human-readable description
    description: String,
    /// Code template for the fix
    code_template: String,
    /// Confidence in this fix (0.0 - 1.0)
    confidence: f32,
}

impl SuggestedFix {
    /// Create a new suggested fix.
    pub fn new(description: impl Into<String>, code_template: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            code_template: code_template.into(),
            confidence: 0.5,
        }
    }

    /// Set confidence score.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Get description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get code template.
    pub fn code_template(&self) -> &str {
        &self.code_template
    }

    /// Get confidence score.
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

/// An ownership inference error pattern.
///
/// Represents a specific C pattern that causes ownership inference failures,
/// along with metadata for curriculum learning and error recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Unique identifier for this pattern
    id: String,
    /// Error category
    error_kind: OwnershipErrorKind,
    /// Human-readable description
    description: String,
    /// Example C code that triggers this error
    c_pattern: Option<String>,
    /// Rust compiler error message (if applicable)
    rust_error: Option<String>,
    /// Suggested fix
    suggested_fix: Option<SuggestedFix>,
    /// Severity level
    severity: ErrorSeverity,
    /// Curriculum learning level (1 = easiest, higher = harder)
    curriculum_level: u8,
    /// Number of times this pattern has been encountered
    occurrence_count: u64,
}

impl ErrorPattern {
    /// Create a new error pattern.
    pub fn new(
        id: impl Into<String>,
        error_kind: OwnershipErrorKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            error_kind,
            description: description.into(),
            c_pattern: None,
            rust_error: None,
            suggested_fix: None,
            severity: ErrorSeverity::Error,
            curriculum_level: 1,
            occurrence_count: 0,
        }
    }

    /// Set the C pattern example.
    pub fn with_c_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.c_pattern = Some(pattern.into());
        self
    }

    /// Set the Rust error message.
    pub fn with_rust_error(mut self, error: impl Into<String>) -> Self {
        self.rust_error = Some(error.into());
        self
    }

    /// Set the suggested fix.
    pub fn with_fix(mut self, fix: SuggestedFix) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Set severity level.
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set curriculum level.
    pub fn with_curriculum_level(mut self, level: u8) -> Self {
        self.curriculum_level = level;
        self
    }

    /// Get pattern ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get error kind.
    pub fn error_kind(&self) -> OwnershipErrorKind {
        self.error_kind
    }

    /// Get description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get C pattern.
    pub fn c_pattern(&self) -> Option<&str> {
        self.c_pattern.as_deref()
    }

    /// Get Rust error.
    pub fn rust_error(&self) -> Option<&str> {
        self.rust_error.as_deref()
    }

    /// Get suggested fix.
    pub fn suggested_fix(&self) -> Option<&SuggestedFix> {
        self.suggested_fix.as_ref()
    }

    /// Get severity.
    pub fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Get curriculum level.
    pub fn curriculum_level(&self) -> u8 {
        self.curriculum_level
    }

    /// Increment occurrence count.
    pub fn record_occurrence(&mut self) {
        self.occurrence_count = self.occurrence_count.saturating_add(1);
    }

    /// Get occurrence count.
    pub fn occurrence_count(&self) -> u64 {
        self.occurrence_count
    }
}

/// Library of error patterns for ownership inference.
///
/// Supports curriculum ordering and pattern matching for error recovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternLibrary {
    /// All patterns indexed by ID
    patterns: std::collections::HashMap<String, ErrorPattern>,
}

impl PatternLibrary {
    /// Create a new empty pattern library.
    pub fn new() -> Self {
        Self {
            patterns: std::collections::HashMap::new(),
        }
    }

    /// Add a pattern to the library.
    pub fn add(&mut self, pattern: ErrorPattern) {
        self.patterns.insert(pattern.id.clone(), pattern);
    }

    /// Get a pattern by ID.
    pub fn get(&self, id: &str) -> Option<&ErrorPattern> {
        self.patterns.get(id)
    }

    /// Get a mutable pattern by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ErrorPattern> {
        self.patterns.get_mut(id)
    }

    /// Get all patterns matching an error kind.
    pub fn get_by_error_kind(&self, kind: OwnershipErrorKind) -> Vec<&ErrorPattern> {
        self.patterns
            .values()
            .filter(|p| p.error_kind == kind)
            .collect()
    }

    /// Get patterns ordered by curriculum level (easiest first).
    pub fn curriculum_ordered(&self) -> Vec<&ErrorPattern> {
        let mut patterns: Vec<_> = self.patterns.values().collect();
        patterns.sort_by_key(|p| p.curriculum_level);
        patterns
    }

    /// Match patterns against a Rust error message.
    pub fn match_rust_error(&self, error_msg: &str) -> Vec<&ErrorPattern> {
        self.patterns
            .values()
            .filter(|p| p.rust_error.as_ref().is_some_and(|e| error_msg.contains(e)))
            .collect()
    }

    /// Get number of patterns.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if library is empty.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Iterate over all patterns.
    pub fn iter(&self) -> impl Iterator<Item = &ErrorPattern> {
        self.patterns.values()
    }
}

// ============================================================================
// DECY-ML-007: DEFAULT PATTERN LIBRARY
// ============================================================================

/// Create a default pattern library with common ownership inference patterns.
///
/// Returns a `PatternLibrary` pre-populated with patterns for all 8 error kinds,
/// organized by curriculum level (simpler patterns first).
///
/// # Example
///
/// ```
/// use decy_ownership::ml_features::default_pattern_library;
///
/// let library = default_pattern_library();
/// assert!(!library.is_empty());
///
/// // Get patterns ordered by curriculum level
/// let ordered = library.curriculum_ordered();
/// for pattern in ordered {
///     println!("{}: {}", pattern.id(), pattern.description());
/// }
/// ```
pub fn default_pattern_library() -> PatternLibrary {
    let mut library = PatternLibrary::new();

    // ========================================================================
    // Level 1: Basic Ownership Patterns (Easiest)
    // ========================================================================

    // DECY-O-001: Pointer Misclassification - malloc → Box
    library.add(
        ErrorPattern::new(
            "malloc-to-box",
            OwnershipErrorKind::PointerMisclassification,
            "Single allocation with malloc should use Box<T>",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(1)
        .with_c_pattern("void *ptr = malloc(sizeof(T))")
        .with_rust_error("E0308")
        .with_fix(SuggestedFix::new(
            "Replace raw pointer with Box",
            "let ptr: Box<T> = Box::new(T::default());",
        )),
    );

    // DECY-O-001: Pointer Misclassification - array → Vec
    library.add(
        ErrorPattern::new(
            "array-to-vec",
            OwnershipErrorKind::PointerMisclassification,
            "Dynamic array allocation should use Vec<T>",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(1)
        .with_c_pattern("T *arr = malloc(n * sizeof(T))")
        .with_rust_error("E0308")
        .with_fix(SuggestedFix::new(
            "Replace pointer array with Vec",
            "let arr: Vec<T> = Vec::with_capacity(n);",
        )),
    );

    // DECY-O-008: Mutability Mismatch - const pointer
    library.add(
        ErrorPattern::new(
            "const-to-immut-ref",
            OwnershipErrorKind::MutabilityMismatch,
            "Const pointer should be immutable reference",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(1)
        .with_c_pattern("const T *ptr")
        .with_rust_error("E0596")
        .with_fix(SuggestedFix::new(
            "Use immutable reference",
            "fn foo(ptr: &T)",
        )),
    );

    // ========================================================================
    // Level 2: Lifetime Patterns (Medium)
    // ========================================================================

    // DECY-O-002: Lifetime Inference Gap - missing lifetime
    library.add(
        ErrorPattern::new(
            "missing-lifetime",
            OwnershipErrorKind::LifetimeInferenceGap,
            "Function returning reference needs lifetime annotation",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(2)
        .with_c_pattern("T* get_field(Struct *s) { return &s->field; }")
        .with_rust_error("E0106")
        .with_fix(SuggestedFix::new(
            "Add lifetime parameter",
            "fn get_field<'a>(s: &'a Struct) -> &'a T",
        )),
    );

    // DECY-O-002: Lifetime Inference Gap - struct lifetime
    library.add(
        ErrorPattern::new(
            "struct-lifetime",
            OwnershipErrorKind::LifetimeInferenceGap,
            "Struct containing reference needs lifetime parameter",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(2)
        .with_c_pattern("struct View { T *data; }")
        .with_rust_error("E0106")
        .with_fix(SuggestedFix::new(
            "Add lifetime to struct",
            "struct View<'a> { data: &'a T }",
        )),
    );

    // DECY-O-006: Array/Slice Mismatch - parameter
    library.add(
        ErrorPattern::new(
            "array-param-to-slice",
            OwnershipErrorKind::ArraySliceMismatch,
            "Array parameter should be slice reference",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(2)
        .with_c_pattern("void process(int arr[], size_t len)")
        .with_rust_error("E0308")
        .with_fix(SuggestedFix::new(
            "Use slice parameter",
            "fn process(arr: &[i32])",
        )),
    );

    // ========================================================================
    // Level 3: Borrow Checker Patterns (Harder)
    // ========================================================================

    // DECY-O-004: Alias Violation - mutable aliasing
    library.add(
        ErrorPattern::new(
            "mutable-aliasing",
            OwnershipErrorKind::AliasViolation,
            "Cannot have multiple mutable references",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(3)
        .with_c_pattern("T *a = ptr; T *b = ptr; *a = x; *b = y;")
        .with_rust_error("E0499")
        .with_fix(SuggestedFix::new(
            "Use single mutable reference or split borrows",
            "// Ensure only one &mut exists at a time",
        )),
    );

    // DECY-O-004: Alias Violation - immut + mut
    library.add(
        ErrorPattern::new(
            "immut-mut-aliasing",
            OwnershipErrorKind::AliasViolation,
            "Cannot have mutable reference while immutable exists",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(3)
        .with_c_pattern("const T *r = ptr; *ptr = x; use(r);")
        .with_rust_error("E0502")
        .with_fix(SuggestedFix::new(
            "End immutable borrow before mutating",
            "let r = &*ptr; use(r); *ptr = x;",
        )),
    );

    // DECY-O-003: Dangling Pointer Risk - use after free
    library.add(
        ErrorPattern::new(
            "use-after-free",
            OwnershipErrorKind::DanglingPointerRisk,
            "Use of pointer after free causes undefined behavior",
        )
        .with_severity(ErrorSeverity::Critical)
        .with_curriculum_level(3)
        .with_c_pattern("free(ptr); use(ptr);")
        .with_rust_error("E0382")
        .with_fix(SuggestedFix::new(
            "Use Option<Box<T>> and take() to consume",
            "let val = box_opt.take(); // Consumes ownership",
        )),
    );

    // DECY-O-003: Dangling Pointer Risk - return local
    library.add(
        ErrorPattern::new(
            "return-local-ref",
            OwnershipErrorKind::DanglingPointerRisk,
            "Returning pointer to local variable is undefined behavior",
        )
        .with_severity(ErrorSeverity::Critical)
        .with_curriculum_level(3)
        .with_c_pattern("int* foo() { int x = 1; return &x; }")
        .with_rust_error("E0515")
        .with_fix(SuggestedFix::new(
            "Return owned value or use parameter lifetime",
            "fn foo() -> i32 { 1 } // Return by value",
        )),
    );

    // ========================================================================
    // Level 4: Resource Management (Advanced)
    // ========================================================================

    // DECY-O-007: Resource Leak - missing free
    library.add(
        ErrorPattern::new(
            "missing-free",
            OwnershipErrorKind::ResourceLeakPattern,
            "Allocated memory not freed causes leak",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(4)
        .with_c_pattern("void* p = malloc(...); return; // leak!")
        .with_fix(SuggestedFix::new(
            "Use RAII with Box/Vec for automatic cleanup",
            "let p = Box::new(...); // Automatically freed",
        )),
    );

    // DECY-O-007: Resource Leak - file handle
    library.add(
        ErrorPattern::new(
            "file-handle-leak",
            OwnershipErrorKind::ResourceLeakPattern,
            "File handle not closed causes resource leak",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(4)
        .with_c_pattern("FILE *f = fopen(...); return; // leak!")
        .with_fix(SuggestedFix::new(
            "Use File type with automatic Drop",
            "let f = File::open(...)?; // Closed on drop",
        )),
    );

    // DECY-O-005: Unsafe Minimization Failure
    library.add(
        ErrorPattern::new(
            "unnecessary-unsafe",
            OwnershipErrorKind::UnsafeMinimizationFailure,
            "Safe alternative exists for this unsafe operation",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(4)
        .with_c_pattern("*(ptr + i) = value; // pointer arithmetic")
        .with_fix(SuggestedFix::new(
            "Use safe slice indexing",
            "slice[i] = value;",
        )),
    );

    // DECY-O-005: Unsafe Minimization - null check
    library.add(
        ErrorPattern::new(
            "null-check-to-option",
            OwnershipErrorKind::UnsafeMinimizationFailure,
            "Null pointer check should use Option<T>",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(4)
        .with_c_pattern("if (ptr != NULL) { use(ptr); }")
        .with_fix(SuggestedFix::new(
            "Use Option<T> with if let or match",
            "if let Some(val) = opt { use(val); }",
        )),
    );

    // ========================================================================
    // Level 5: Complex Patterns (Expert)
    // ========================================================================

    // DECY-O-004: Alias Violation - self-referential
    library.add(
        ErrorPattern::new(
            "self-referential-struct",
            OwnershipErrorKind::AliasViolation,
            "Self-referential struct needs Pin or unsafe",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(5)
        .with_c_pattern("struct Node { struct Node *next; int data; }")
        .with_rust_error("E0597")
        .with_fix(SuggestedFix::new(
            "Use Box for indirection or Pin for self-reference",
            "struct Node { next: Option<Box<Node>>, data: i32 }",
        )),
    );

    // DECY-O-002: Lifetime Inference - multiple lifetimes
    library.add(
        ErrorPattern::new(
            "multiple-lifetimes",
            OwnershipErrorKind::LifetimeInferenceGap,
            "Function with multiple reference params needs explicit lifetimes",
        )
        .with_severity(ErrorSeverity::Error)
        .with_curriculum_level(5)
        .with_c_pattern("T* pick(T *a, T *b, int cond)")
        .with_rust_error("E0106")
        .with_fix(SuggestedFix::new(
            "Add explicit lifetime bounds",
            "fn pick<'a>(a: &'a T, b: &'a T, cond: bool) -> &'a T",
        )),
    );

    // DECY-O-008: Mutability - interior mutability
    library.add(
        ErrorPattern::new(
            "interior-mutability",
            OwnershipErrorKind::MutabilityMismatch,
            "Mutation through shared reference needs Cell/RefCell",
        )
        .with_severity(ErrorSeverity::Warning)
        .with_curriculum_level(5)
        .with_c_pattern("void inc(Counter *c) { c->count++; } // called via const ptr")
        .with_rust_error("E0596")
        .with_fix(SuggestedFix::new(
            "Use Cell<T> or RefCell<T> for interior mutability",
            "struct Counter { count: Cell<i32> }",
        )),
    );

    library
}

#[cfg(test)]
#[path = "ml_features_coverage_tests.rs"]
mod ml_features_coverage_tests;
