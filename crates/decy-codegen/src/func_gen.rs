//! Function, struct, enum, typedef, constant, and global variable code generation.
//!
//! Contains all methods related to generating Rust code from HIR top-level
//! declarations: function signatures, function bodies with ownership analysis,
//! struct/enum definitions, typedefs, constants, and global variables.

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};
use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};

impl CodeGenerator {
    /// Generate a function signature from HIR.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType};
    ///
    /// let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    /// let codegen = CodeGenerator::new();
    /// let sig = codegen.generate_signature(&func);
    ///
    /// assert_eq!(sig, "fn test()");
    /// ```
    pub fn generate_signature(&self, func: &HirFunction) -> String {
        // DECY-076 GREEN: Generate lifetime annotations using LifetimeAnnotator
        use decy_ownership::lifetime_gen::LifetimeAnnotator;
        let lifetime_annotator = LifetimeAnnotator::new();
        let annotated_sig = lifetime_annotator.annotate_function(func);

        // DECY-241: Rename functions that conflict with Rust macros/keywords
        let safe_name = match func.name() {
            "write" => "c_write", // Conflicts with Rust's write! macro
            "read" => "c_read",   // Conflicts with Rust's read
            "type" => "c_type",   // Rust keyword
            "match" => "c_match", // Rust keyword
            "self" => "c_self",   // Rust keyword
            "in" => "c_in",       // Rust keyword
            name => name,
        };
        let mut sig = format!("fn {}", safe_name);

        // Add lifetime parameters if needed
        let lifetime_syntax = lifetime_annotator.generate_lifetime_syntax(&annotated_sig.lifetimes);
        sig.push_str(&lifetime_syntax);

        // DECY-096: Detect void* parameters for generic transformation
        use decy_analyzer::void_ptr_analysis::{TypeConstraint, VoidPtrAnalyzer};
        let void_analyzer = VoidPtrAnalyzer::new();
        let void_patterns = void_analyzer.analyze(func);

        // DECY-168: Only consider patterns with actual constraints/types as "real" void* usage
        // Empty body functions (stubs) will have patterns but no constraints
        let has_real_void_usage = void_patterns
            .iter()
            .any(|vp| !vp.constraints.is_empty() || !vp.inferred_types.is_empty());

        // DECY-097: Collect trait bounds from all void* patterns
        let mut trait_bounds: Vec<&str> = Vec::new();
        for pattern in &void_patterns {
            for constraint in &pattern.constraints {
                let bound = match constraint {
                    TypeConstraint::PartialOrd => "PartialOrd",
                    TypeConstraint::PartialEq => "PartialEq",
                    TypeConstraint::Clone => "Clone",
                    TypeConstraint::Copy => "Copy",
                    _ => continue,
                };
                if !trait_bounds.contains(&bound) {
                    trait_bounds.push(bound);
                }
            }
        }

        // Add generic type parameter with trait bounds if function has void* params with real usage
        // DECY-168: Don't add <T> for stub functions without body analysis
        if has_real_void_usage {
            if trait_bounds.is_empty() {
                sig.push_str("<T>");
            } else {
                sig.push_str(&format!("<T: {}>", trait_bounds.join(" + ")));
            }
        }

        // DECY-072 GREEN: Detect array parameters using ownership analysis
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        // DECY-084 GREEN: Detect output parameters for transformation
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let output_detector = OutputParamDetector::new();
        let output_params = output_detector.detect(func);

        // Track which parameters are length parameters to skip them
        let mut skip_params = std::collections::HashSet::new();

        // DECY-084: Track output parameters to skip and use for return type
        let mut output_param_type: Option<HirType> = None;
        let mut output_is_fallible = false;
        for op in &output_params {
            if op.kind == ParameterKind::Output {
                skip_params.insert(op.name.clone());
                output_is_fallible = op.is_fallible;
                // Get the output parameter's inner type (pointer target)
                if let Some(param) = func.parameters().iter().find(|p| p.name() == op.name) {
                    if let HirType::Pointer(inner) = param.param_type() {
                        output_param_type = Some((**inner).clone());
                    }
                }
            }
        }

        // First pass: identify array parameters and their associated length parameters
        // DECY-113: Only skip params with length-like names to avoid removing non-length params
        // DECY-162: Don't skip length param if array uses pointer arithmetic (stays as raw pointer)
        for (idx, param) in func.parameters().iter().enumerate() {
            if let Some(true) = graph.is_array_parameter(param.name()) {
                // DECY-162: Don't skip length param if array uses pointer arithmetic
                // Raw pointers don't have .len(), so we need to keep the size param
                if self.uses_pointer_arithmetic(func, param.name()) {
                    continue; // Skip adding length param to skip_params
                }

                // This is an array parameter - mark the next param as length param to skip
                // but only if it has a length-like name
                if idx + 1 < func.parameters().len() {
                    let next_param = &func.parameters()[idx + 1];
                    if matches!(next_param.param_type(), HirType::Int) {
                        let param_name = next_param.name().to_lowercase();
                        // Only skip if the name suggests it's a length/size parameter
                        if param_name.contains("len")
                            || param_name.contains("size")
                            || param_name.contains("count")
                            || param_name == "n"
                            || param_name == "num"
                        {
                            skip_params.insert(next_param.name().to_string());
                        }
                    }
                }
            }
        }

        // Generate parameters with lifetime annotations
        sig.push('(');
        let params: Vec<String> = annotated_sig
            .parameters
            .iter()
            .filter_map(|p| {
                if skip_params.contains(&p.name) {
                    return None;
                }
                self.generate_signature_param(p, func, &graph, &void_patterns)
            })
            .collect();
        sig.push_str(&params.join(", "));
        sig.push(')');

        // Generate return type
        self.append_signature_return_type(
            &mut sig,
            func,
            output_param_type.as_ref(),
            output_is_fallible,
            &annotated_sig,
        );

        sig
    }

    /// Generate a single parameter for a function signature.
    fn generate_signature_param(
        &self,
        p: &decy_ownership::lifetime_gen::AnnotatedParameter,
        func: &HirFunction,
        graph: &decy_ownership::dataflow::DataflowGraph,
        void_patterns: &[decy_analyzer::void_ptr_analysis::VoidPtrInfo],
    ) -> Option<String> {
        // Check if this is an array parameter
        let is_array = graph.is_array_parameter(&p.name).unwrap_or(false);

        // DECY-161: Array params with pointer arithmetic must stay as raw pointers
        // Slices don't support arr++ or arr + n, so check for pointer arithmetic first
        let uses_ptr_arithmetic = self.uses_pointer_arithmetic(func, &p.name);

        if is_array && !uses_ptr_arithmetic {
            // Transform to slice parameter (only if no pointer arithmetic)
            // Find the original parameter to get the HirType
            if let Some(orig_param) =
                func.parameters().iter().find(|fp| fp.name() == p.name)
            {
                let is_mutable = self.is_parameter_modified(func, &p.name);
                let slice_type =
                    self.pointer_to_slice_type(orig_param.param_type(), is_mutable);
                // For slices, don't add 'mut' prefix (slices themselves aren't reassigned)
                Some(format!("{}: {}", p.name, slice_type))
            } else {
                None
            }
        } else {
            // DECY-086: Check if this is an array parameter that should become a slice
            // In C, `int arr[10]` as a parameter decays to a pointer, so we use slice
            if let Some(orig_param) =
                func.parameters().iter().find(|fp| fp.name() == p.name)
            {
                if let HirType::Array { element_type, .. } = orig_param.param_type() {
                    // Fixed-size array parameter → slice reference
                    let is_mutable = self.is_parameter_modified(func, &p.name);
                    let element_str = Self::map_type(element_type);
                    if is_mutable {
                        return Some(format!("{}: &mut [{}]", p.name, element_str));
                    } else {
                        return Some(format!("{}: &[{}]", p.name, element_str));
                    }
                }
            }
            // DECY-111: Check if this is a pointer parameter that should become a reference
            // DECY-123: Skip transformation if pointer arithmetic is used
            if let Some(orig_param) =
                func.parameters().iter().find(|fp| fp.name() == p.name)
            {
                // DECY-135: const char* → &str transformation
                // DECY-138: Add mut for string iteration patterns (param reassignment)
                // Must check BEFORE other pointer transformations
                if orig_param.is_const_char_pointer() {
                    return Some(format!("mut {}: &str", p.name));
                }

                if let HirType::Pointer(inner) = orig_param.param_type() {
                    return Some(self.generate_pointer_param(
                        &p.name, inner, func, void_patterns,
                    ));
                }
            }
            // Regular parameter with lifetime annotation
            let type_str = self.annotated_type_to_string(&p.param_type);
            // In C, parameters are mutable by default (can be reassigned)
            Some(format!("mut {}: {}", p.name, type_str))
        }
    }

    /// Generate a pointer parameter representation (reference, raw pointer, slice, or generic).
    fn generate_pointer_param(
        &self,
        name: &str,
        inner: &HirType,
        func: &HirFunction,
        void_patterns: &[decy_analyzer::void_ptr_analysis::VoidPtrInfo],
    ) -> String {
        use decy_analyzer::void_ptr_analysis::TypeConstraint;

        // DECY-096: void* param becomes generic &T or &mut T
        // DECY-168: Only apply generic transformation if we found an actual pattern
        // for this specific parameter WITH real constraints (from body analysis).
        // Otherwise keep as raw pointer *mut ().
        if matches!(inner, HirType::Void) {
            // Look for a void pattern specifically for this parameter
            // that has actual constraints (indicating real usage in body)
            let void_pattern = void_patterns.iter().find(|vp| {
                vp.param_name == name
                    && (!vp.constraints.is_empty()
                        || !vp.inferred_types.is_empty())
            });

            if let Some(pattern) = void_pattern {
                // Found actual usage pattern - apply generic transformation
                let is_mutable = pattern.constraints.contains(&TypeConstraint::Mutable);
                if is_mutable {
                    return format!("{}: &mut T", name);
                } else {
                    return format!("{}: &T", name);
                }
            } else {
                // DECY-168: No pattern with real constraints found - keep as raw pointer
                // This is important for stdlib stubs (realloc, memcpy, etc.)
                return format!("{}: *mut ()", name);
            }
        }
        // DECY-134: Check for string iteration pattern FIRST
        // char* with pointer arithmetic → slice instead of raw pointer
        if self.is_string_iteration_param(func, name) {
            // Transform to slice for safe string iteration
            let is_mutable = self.is_parameter_deref_modified(func, name);
            if is_mutable {
                return format!("{}: &mut [u8]", name);
            } else {
                return format!("{}: &[u8]", name);
            }
        }
        // DECY-123: Don't transform to reference if pointer arithmetic is used
        // (e.g., ptr = ptr + 1) - keep as raw pointer
        if self.uses_pointer_arithmetic(func, name) {
            // Keep as raw pointer - will need unsafe blocks
            // DECY-124: Add mut since the pointer is reassigned
            let inner_type = Self::map_type(inner);
            return format!("mut {}: *mut {}", name, inner_type);
        }
        // Transform pointer param to mutable reference
        // Check if the param is modified in the function body
        let is_mutable = self.is_parameter_deref_modified(func, name);
        let inner_type = Self::map_type(inner);
        if is_mutable {
            format!("{}: &mut {}", name, inner_type)
        } else {
            // Read-only pointer becomes immutable reference
            format!("{}: &{}", name, inner_type)
        }
    }

    /// Append return type to signature string.
    fn append_signature_return_type(
        &self,
        sig: &mut String,
        func: &HirFunction,
        output_param_type: Option<&HirType>,
        output_is_fallible: bool,
        annotated_sig: &AnnotatedSignature,
    ) {
        // Special handling for main function (DECY-AUDIT-001)
        // C's int main() must become Rust's fn main() (no return type)
        // Rust's entry point returns () and uses std::process::exit(N) for exit codes
        if func.name() == "main" && matches!(func.return_type(), HirType::Int) {
            return;
        }

        // DECY-084 GREEN: Generate return type considering output parameters
        // Priority: output param type > original return type
        if let Some(out_type) = output_param_type {
            let out_type_str = Self::map_type(out_type);
            if output_is_fallible {
                sig.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                sig.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check if function returns malloc'd array → use Vec<T>
            if let Some(vec_element_type) = self.detect_vec_return(func) {
                let element_type_str = Self::map_type(&vec_element_type);
                sig.push_str(&format!(" -> Vec<{}>", element_type_str));
            } else {
                // Generate return type with lifetime annotation (skip for void)
                if !matches!(&annotated_sig.return_type, AnnotatedType::Simple(HirType::Void)) {
                    let return_type_str = self.annotated_type_to_string(&annotated_sig.return_type);
                    sig.push_str(&format!(" -> {}", return_type_str));
                }
            }
        }
    }

    /// DECY-142: Check if function returns a malloc-allocated array.
    /// Returns Some(element_type) if the function allocates with malloc and returns it.
    /// This pattern should use Vec<T> return type instead of *mut T.
    pub(crate) fn detect_vec_return(&self, func: &HirFunction) -> Option<HirType> {
        // Only applies to functions returning pointer types
        let return_type = func.return_type();
        let element_type = match return_type {
            HirType::Pointer(inner) => inner.as_ref().clone(),
            _ => return None,
        };

        // Look for pattern: var = malloc(...); return var;
        // or: return malloc(...);
        let mut malloc_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

        for stmt in func.body() {
            // Track variables assigned from malloc
            if let HirStatement::VariableDeclaration {
                name, initializer: Some(init_expr), ..
            } = stmt
            {
                if Self::is_malloc_call(init_expr) {
                    malloc_vars.insert(name.clone());
                }
            }

            // Check return statements
            if let HirStatement::Return(Some(ret_expr)) = stmt {
                // Direct return of malloc
                if Self::is_malloc_call(ret_expr) {
                    return Some(element_type);
                }
                // Return of a variable that was assigned from malloc
                if let HirExpression::Variable(var_name) = ret_expr {
                    if malloc_vars.contains(var_name) {
                        return Some(element_type);
                    }
                }
            }
        }

        None
    }

    /// Helper to check if an expression is ANY malloc or calloc call (including through casts).
    /// DECY-220: This is used for type annotation transformation (*mut T → Vec<T>).
    /// Unlike `is_malloc_call`, this returns true for ANY malloc/calloc, not just array patterns.
    pub(crate) fn is_any_malloc_or_calloc(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::Malloc { .. } => true,
            HirExpression::Calloc { .. } => true,
            HirExpression::FunctionCall { function, .. }
                if function == "malloc" || function == "calloc" =>
            {
                true
            }
            // DECY-220: Check through cast expressions (e.g., (int*)malloc(...))
            HirExpression::Cast { expr: inner, .. } => Self::is_any_malloc_or_calloc(inner),
            _ => false,
        }
    }

    /// Helper to check if an expression is a malloc call for ARRAY allocation.
    /// DECY-142: Only returns true for array allocations (malloc(n * sizeof(T))),
    /// not single struct allocations (malloc(sizeof(T))).
    fn is_malloc_call(expr: &HirExpression) -> bool {
        match expr {
            HirExpression::FunctionCall { function, arguments, .. } if function == "malloc" => {
                // Check if this is an array allocation: malloc(n * sizeof(T))
                // Single struct allocation: malloc(sizeof(T)) should NOT match
                if arguments.len() == 1 {
                    Self::is_array_allocation_size(&arguments[0])
                } else {
                    false
                }
            }
            HirExpression::Malloc { size } => {
                // Check if this is an array allocation
                Self::is_array_allocation_size(size)
            }
            // DECY-142: Check through cast expressions (e.g., (int*)malloc(...))
            HirExpression::Cast { expr: inner, .. } => Self::is_malloc_call(inner),
            _ => false,
        }
    }

    /// Check if a malloc size expression indicates array allocation (n * sizeof(T))
    /// vs single struct allocation (sizeof(T) or constant).
    fn is_array_allocation_size(size_expr: &HirExpression) -> bool {
        match size_expr {
            // n * sizeof(T) pattern - this is array allocation
            HirExpression::BinaryOp { op: decy_hir::BinaryOperator::Multiply, .. } => true,
            // sizeof(T) alone - this is single struct allocation, NOT array
            HirExpression::Sizeof { .. } => false,
            // Constant - likely single allocation
            HirExpression::IntLiteral(_) => false,
            // Variable could be array size, but be conservative
            HirExpression::Variable(_) => false,
            // Recurse through casts
            HirExpression::Cast { expr: inner, .. } => Self::is_array_allocation_size(inner),
            // Other expressions - be conservative, assume not array
            _ => false,
        }
    }

    /// Check if a parameter is modified in the function body (DECY-072 GREEN).
    ///
    /// Used to determine whether to use `&[T]` or `&mut [T]` for array parameters.
    fn is_parameter_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        // Check if the parameter is used in any assignment statements
        for stmt in func.body() {
            if self.statement_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Check if a pointer parameter is dereferenced and modified (DECY-111 GREEN).
    ///
    /// Used to determine whether to use `&T` or `&mut T` for pointer parameters.
    /// Returns true if the parameter is used in:
    /// - `*ptr = value;` (DerefAssignment)
    /// - `ptr[i] = value;` (ArrayIndexAssignment with pointer)
    pub(crate) fn is_parameter_deref_modified(&self, func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_deref_modifies_variable(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement deref-modifies a variable (DECY-111 GREEN).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_deref_modifies_variable(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::DerefAssignment { target, .. } => {
                // Check if this is *ptr = value where ptr is our variable
                if let HirExpression::Variable(name) = target {
                    return name == var_name;
                }
                false
            }
            HirStatement::ArrayIndexAssignment { array, .. } => {
                // Check if this is ptr[i] = value where ptr is our variable
                if let HirExpression::Variable(name) = &**array {
                    return name == var_name;
                }
                false
            }
            HirStatement::Assignment { .. } => {
                // Regular variable assignment (src = src + 1) does NOT modify *src
                // Only DerefAssignment (*src = value) modifies the pointed-to value
                false
            }
            HirStatement::If { then_block, else_block, .. } => {
                then_block.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_deref_modifies_variable(s, var_name))
            }
            _ => false,
        }
    }

    /// Check if a parameter uses pointer arithmetic, is reassigned, or compared to NULL (DECY-123, DECY-137).
    ///
    /// Used to determine whether a pointer parameter should remain a raw pointer
    /// instead of being transformed to a reference.
    /// Returns true if the parameter is used in:
    /// - `ptr = ptr + n;` (pointer arithmetic assignment)
    /// - `ptr = ptr - n;` (pointer arithmetic assignment)
    /// - `ptr += n;` or `ptr -= n;` (compound pointer arithmetic)
    /// - `ptr = ptr->field;` (DECY-137: linked list traversal pattern)
    /// - `ptr = other_ptr;` (any pointer reassignment)
    /// - `ptr != 0` or `ptr == 0` (DECY-137: NULL comparison - Rust refs can't be null)
    ///
    /// References in Rust cannot be reassigned or null, so any pointer param that is
    /// reassigned or NULL-checked must remain as a raw pointer.
    pub(crate) fn uses_pointer_arithmetic(&self, func: &HirFunction, param_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_uses_pointer_arithmetic(stmt, param_name) {
                return true;
            }
            // DECY-137: Also check for NULL comparisons in conditions
            if self.statement_uses_null_comparison(stmt, param_name) {
                return true;
            }
        }
        false
    }

    /// Check if a statement contains NULL comparison for a variable (DECY-137).
    ///
    /// If a pointer is compared to NULL (0), it should stay as raw pointer
    /// because Rust references can never be null.
    #[allow(clippy::only_used_in_recursion)]
    fn statement_uses_null_comparison(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::If { condition, then_block, else_block, .. } => {
                // Check condition for NULL comparison
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                // Recursively check nested statements
                then_block.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
                    })
            }
            HirStatement::While { condition, body, .. } => {
                if self.expression_compares_to_null(condition, var_name) {
                    return true;
                }
                body.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            HirStatement::For { condition, body, .. } => {
                if let Some(cond) = condition {
                    if self.expression_compares_to_null(cond, var_name) {
                        return true;
                    }
                }
                body.iter().any(|s| self.statement_uses_null_comparison(s, var_name))
            }
            _ => false,
        }
    }

    /// Check if an expression compares a variable to NULL (0).
    fn expression_compares_to_null(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right } => {
                if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                    // Check: var == 0 or var != 0
                    if let HirExpression::Variable(name) = &**left {
                        if name == var_name
                            && matches!(
                                **right,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                    // Check: 0 == var or 0 != var
                    if let HirExpression::Variable(name) = &**right {
                        if name == var_name
                            && matches!(
                                **left,
                                HirExpression::IntLiteral(0) | HirExpression::NullLiteral
                            )
                        {
                            return true;
                        }
                    }
                }
                // Recursively check nested expressions (e.g., in logical AND/OR)
                self.expression_compares_to_null(left, var_name)
                    || self.expression_compares_to_null(right, var_name)
            }
            _ => false,
        }
    }

    /// Recursively check if a statement uses pointer arithmetic or reassigns a variable (DECY-123, DECY-137).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_uses_pointer_arithmetic(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::Assignment { target, value } => {
                // DECY-137: Any assignment to the pointer parameter means it must stay as raw pointer
                // This catches:
                // - ptr = ptr + n (pointer arithmetic)
                // - ptr = ptr->next (linked list traversal)
                // - ptr = other_ptr (general reassignment)
                //
                // References cannot be reassigned, only raw pointers can.
                if target == var_name {
                    // Check if this is pointer arithmetic (ptr = ptr + n or ptr = ptr - n)
                    if let HirExpression::BinaryOp { op, left, .. } = value {
                        if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                            if let HirExpression::Variable(name) = &**left {
                                if name == var_name {
                                    return true;
                                }
                            }
                        }
                    }

                    // DECY-137: Check for field access reassignment (ptr = ptr->field)
                    // This is the linked list traversal pattern: head = head->next
                    if let HirExpression::PointerFieldAccess { pointer, .. } = value {
                        if let HirExpression::Variable(name) = &**pointer {
                            if name == var_name {
                                return true;
                            }
                        }
                    }

                    // DECY-137: Check for any other pointer reassignment
                    // If ptr is assigned from another variable or expression, it needs
                    // to stay as raw pointer. However, we need to be careful not to
                    // flag initialization (which happens at declaration, not assignment).
                    // For now, flag field access from ANY pointer as reassignment.
                    if matches!(value, HirExpression::PointerFieldAccess { .. }) {
                        return true;
                    }
                }
                false
            }
            HirStatement::If { then_block, else_block, .. } => {
                then_block.iter().any(|s| self.statement_uses_pointer_arithmetic(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_uses_pointer_arithmetic(s, var_name))
                    })
            }
            // DECY-164: Check for post/pre increment/decrement on the variable
            HirStatement::Expression(expr) => {
                Self::expression_uses_pointer_arithmetic_static(expr, var_name)
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_uses_pointer_arithmetic(s, var_name))
            }
            _ => false,
        }
    }

    /// DECY-164: Check if an expression uses pointer arithmetic on a variable.
    /// Catches str++, ++str, str--, --str patterns.
    fn expression_uses_pointer_arithmetic_static(expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::PostIncrement { operand }
            | HirExpression::PreIncrement { operand }
            | HirExpression::PostDecrement { operand }
            | HirExpression::PreDecrement { operand } => {
                matches!(&**operand, HirExpression::Variable(name) if name == var_name)
            }
            _ => false,
        }
    }

    /// DECY-134b: Get all string iteration params for a function.
    ///
    /// Returns a list of (param_index, is_mutable) for each char* param that uses pointer arithmetic.
    /// Used by decy-core to build string_iter_funcs info for call site transformation.
    pub fn get_string_iteration_params(&self, func: &HirFunction) -> Vec<(usize, bool)> {
        func.parameters()
            .iter()
            .enumerate()
            .filter_map(|(i, param)| {
                if self.is_string_iteration_param(func, param.name()) {
                    let is_mutable = self.is_parameter_deref_modified(func, param.name());
                    Some((i, is_mutable))
                } else {
                    None
                }
            })
            .collect()
    }

    /// DECY-134: Check if a char* parameter is used in a string iteration pattern.
    ///
    /// String iteration pattern: char* with pointer arithmetic in a loop (while (*s) { s++; })
    /// These should be transformed to slice + index for safe Rust.
    /// DECY-164: Skip if function uses pointer subtraction (e.g., str - start for length calculation).
    pub(crate) fn is_string_iteration_param(&self, func: &HirFunction, param_name: &str) -> bool {
        // Must be a char pointer (Pointer(Char))
        let is_char_ptr = func.parameters().iter().any(|p| {
            p.name() == param_name
                && matches!(p.param_type(), HirType::Pointer(inner) if matches!(&**inner, HirType::Char))
        });

        if !is_char_ptr {
            return false;
        }

        // DECY-164: Don't apply string iteration transformation if there's pointer subtraction
        // Pointer subtraction (str - start) requires raw pointers, can't use slices
        if self.function_uses_pointer_subtraction(func, param_name) {
            return false;
        }

        // Must use pointer arithmetic
        self.uses_pointer_arithmetic(func, param_name)
    }

    /// DECY-164: Check if a function uses pointer subtraction involving a variable.
    /// Pattern: var - other_ptr (e.g., str - start for calculating string length)
    fn function_uses_pointer_subtraction(&self, func: &HirFunction, var_name: &str) -> bool {
        for stmt in func.body() {
            if self.statement_uses_pointer_subtraction(stmt, var_name) {
                return true;
            }
        }
        false
    }

    /// DECY-164: Check if a statement uses pointer subtraction involving a variable.
    fn statement_uses_pointer_subtraction(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::Return(Some(expr)) => {
                self.expression_uses_pointer_subtraction(expr, var_name)
            }
            HirStatement::Assignment { value, .. } => {
                self.expression_uses_pointer_subtraction(value, var_name)
            }
            HirStatement::VariableDeclaration { initializer, .. } => initializer
                .as_ref()
                .map(|e| self.expression_uses_pointer_subtraction(e, var_name))
                .unwrap_or(false),
            HirStatement::If { condition, then_block, else_block, .. } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || then_block
                        .iter()
                        .any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
                    })
            }
            HirStatement::While { condition, body } => {
                self.expression_uses_pointer_subtraction(condition, var_name)
                    || body.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
            }
            HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_uses_pointer_subtraction(s, var_name))
            }
            _ => false,
        }
    }

    /// DECY-164: Check if an expression uses pointer subtraction involving a variable.
    fn expression_uses_pointer_subtraction(&self, expr: &HirExpression, var_name: &str) -> bool {
        match expr {
            HirExpression::BinaryOp { op, left, right } => {
                // Check for var - other_ptr pattern
                if matches!(op, BinaryOperator::Subtract) {
                    if let HirExpression::Variable(name) = &**left {
                        if name == var_name {
                            return true;
                        }
                    }
                    if let HirExpression::Variable(name) = &**right {
                        if name == var_name {
                            return true;
                        }
                    }
                }
                // Recursively check subexpressions
                self.expression_uses_pointer_subtraction(left, var_name)
                    || self.expression_uses_pointer_subtraction(right, var_name)
            }
            HirExpression::Dereference(inner) => {
                self.expression_uses_pointer_subtraction(inner, var_name)
            }
            HirExpression::Cast { expr, .. } => {
                self.expression_uses_pointer_subtraction(expr, var_name)
            }
            _ => false,
        }
    }

    /// Recursively check if a statement modifies a variable (DECY-072 GREEN).
    #[allow(clippy::only_used_in_recursion)]
    fn statement_modifies_variable(&self, stmt: &HirStatement, var_name: &str) -> bool {
        match stmt {
            HirStatement::ArrayIndexAssignment { array, .. } => {
                // Check if this is arr[i] = value where arr is our variable
                if let HirExpression::Variable(name) = &**array {
                    return name == var_name;
                }
                false
            }
            HirStatement::DerefAssignment { target, .. } => {
                // Check if this is *ptr = value where ptr is our variable
                if let HirExpression::Variable(name) = target {
                    return name == var_name;
                }
                false
            }
            HirStatement::If { then_block, else_block, .. } => {
                then_block.iter().any(|s| self.statement_modifies_variable(s, var_name))
                    || else_block.as_ref().is_some_and(|blk| {
                        blk.iter().any(|s| self.statement_modifies_variable(s, var_name))
                    })
            }
            HirStatement::While { body, .. } | HirStatement::For { body, .. } => {
                body.iter().any(|s| self.statement_modifies_variable(s, var_name))
            }
            _ => false,
        }
    }

    /// Convert a pointer type to a slice type (DECY-072 GREEN).
    ///
    /// Transforms `*mut T` or `*const T` to `&\[T]` or `&mut \[T]`.
    fn pointer_to_slice_type(&self, ptr_type: &HirType, is_mutable: bool) -> String {
        if let HirType::Pointer(inner) = ptr_type {
            let element_type = Self::map_type(inner);
            if is_mutable {
                format!("&mut [{}]", element_type)
            } else {
                format!("&[{}]", element_type)
            }
        } else {
            // Fallback: not a pointer, use normal mapping
            Self::map_type(ptr_type)
        }
    }

    /// Transform length parameter references to array.len() calls (DECY-072 GREEN).
    ///
    /// Replaces variable references like `len` with `arr.len()` in generated code.
    pub(crate) fn transform_length_refs(
        &self,
        code: &str,
        length_to_array: &std::collections::HashMap<String, String>,
    ) -> String {
        let mut result = code.to_string();

        // Replace each length parameter reference with corresponding array.len() call
        for (length_param, array_param) in length_to_array {
            // Match the length parameter as a standalone identifier
            // Use word boundaries to avoid partial matches
            // Common patterns: "return len", "x + len", "len)", etc.
            let patterns = vec![
                (
                    format!("return {}", length_param),
                    format!("return {}.len() as i32", array_param),
                ),
                (format!("{} ", length_param), format!("{}.len() as i32 ", array_param)),
                (format!("{})", length_param), format!("{}.len() as i32)", array_param)),
                (format!("{},", length_param), format!("{}.len() as i32,", array_param)),
                (format!("{}]", length_param), format!("{}.len() as i32]", array_param)),
                (length_param.clone() + "}", array_param.clone() + ".len() as i32}"),
                (format!("{};", length_param), format!("{}.len() as i32;", array_param)),
            ];

            for (pattern, replacement) in patterns {
                result = result.replace(&pattern, &replacement);
            }
        }

        result
    }

}
