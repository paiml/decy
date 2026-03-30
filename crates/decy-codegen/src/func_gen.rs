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
    fn detect_vec_return(&self, func: &HirFunction) -> Option<HirType> {
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
    fn is_parameter_deref_modified(&self, func: &HirFunction, param_name: &str) -> bool {
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
    fn uses_pointer_arithmetic(&self, func: &HirFunction, param_name: &str) -> bool {
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
    fn is_string_iteration_param(&self, func: &HirFunction, param_name: &str) -> bool {
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
    fn transform_length_refs(
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

    /// Generate a function signature with lifetime annotations.
    ///
    /// Takes an `AnnotatedSignature` with lifetime information and generates
    /// the complete Rust function signature including lifetime parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedParameter, AnnotatedType, LifetimeParam};
    /// use decy_hir::HirType;
    ///
    /// let sig = AnnotatedSignature {
    ///     name: "get_first".to_string(),
    ///     lifetimes: vec![LifetimeParam::standard(0)], // 'a
    ///     parameters: vec![
    ///         AnnotatedParameter {
    ///             name: "items".to_string(),
    ///             param_type: AnnotatedType::Reference {
    ///                 inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///                 mutable: false,
    ///                 lifetime: Some(LifetimeParam::standard(0)),
    ///             },
    ///         },
    ///     ],
    ///     return_type: AnnotatedType::Reference {
    ///         inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///         mutable: false,
    ///         lifetime: Some(LifetimeParam::standard(0)),
    ///     },
    /// };
    ///
    /// let codegen = CodeGenerator::new();
    /// let rust_sig = codegen.generate_annotated_signature(&sig);
    ///
    /// assert!(rust_sig.contains("<'a>"));
    /// assert!(rust_sig.contains("&'a i32"));
    /// ```
    pub fn generate_annotated_signature(&self, sig: &AnnotatedSignature) -> String {
        self.generate_annotated_signature_with_func(sig, None)
    }

    /// Generate a function signature from an annotated signature with optional function body access.
    ///
    /// When `func` is provided, pointer arithmetic detection is enabled (DECY-123).
    /// DECY-084: Also detects output parameters for transformation to return values.
    pub fn generate_annotated_signature_with_func(
        &self,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
    ) -> String {
        // DECY-241: Rename functions that conflict with Rust macros/keywords
        let safe_name = match sig.name.as_str() {
            "write" => "c_write",
            "read" => "c_read",
            "type" => "c_type",
            "match" => "c_match",
            "self" => "c_self",
            "in" => "c_in",
            name => name,
        };
        let mut result = format!("fn {}", safe_name);

        // DECY-084/085: Detect output parameters for transformation
        let (skip_output_params, output_param_types, output_is_fallible) =
            Self::detect_output_params(func);

        // DECY-072: Check if we have any non-slice reference parameters that need lifetimes
        // Slices have elided lifetimes and don't need explicit lifetime parameters
        let has_non_slice_references = sig.parameters.iter().any(|p| {
            match &p.param_type {
                AnnotatedType::Reference { inner, .. } => {
                    // Check if this is NOT a slice (slice = Reference to Array with size=None)
                    !matches!(&**inner, AnnotatedType::Simple(HirType::Array { size: None, .. }))
                }
                _ => false,
            }
        });

        // Add lifetime parameters only if we have non-slice references
        if !sig.lifetimes.is_empty() && has_non_slice_references {
            let lifetime_params: Vec<String> =
                sig.lifetimes.iter().map(|lt| lt.name.clone()).collect();
            result.push_str(&format!("<{}>", lifetime_params.join(", ")));
        }

        // Add function parameters (DECY-084: filter out output params)
        result.push('(');
        let params: Vec<String> = sig
            .parameters
            .iter()
            .filter(|p| !skip_output_params.contains(&p.name))
            .map(|p| self.generate_annotated_param(p, func))
            .collect();
        result.push_str(&params.join(", "));
        result.push(')');

        // Generate return type
        self.append_annotated_return_type(
            &mut result,
            sig,
            func,
            &output_param_types,
            output_is_fallible,
        );

        result
    }

    /// Detect output parameters from a function for signature transformation.
    /// Returns (skip_set, output_types, is_fallible).
    fn detect_output_params(
        func: Option<&HirFunction>,
    ) -> (std::collections::HashSet<String>, Vec<HirType>, bool) {
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let mut skip_output_params = std::collections::HashSet::new();
        let mut output_param_types: Vec<HirType> = Vec::new();
        let mut output_is_fallible = false;

        if let Some(f) = func {
            let output_detector = OutputParamDetector::new();
            let output_params = output_detector.detect(f);

            // Count non-pointer parameters (inputs)
            let input_param_count = f
                .parameters()
                .iter()
                .filter(|p| !matches!(p.param_type(), HirType::Pointer(_)))
                .count();

            // Count potential output params for heuristic
            let output_param_count =
                output_params.iter().filter(|op| op.kind == ParameterKind::Output).count();

            for op in &output_params {
                if op.kind == ParameterKind::Output {
                    // Heuristic: Only treat as output param if:
                    // 1. There are other input parameters (output is derived from inputs)
                    // 2. Or, the name suggests it's an output (result, out, output, ret, etc.)
                    // 3. DECY-085: Or, there are multiple output params (void func with multiple outs)
                    let is_output_name = Self::is_output_param_name(&op.name);

                    if input_param_count > 0 || is_output_name || output_param_count >= 2 {
                        skip_output_params.insert(op.name.clone());
                        output_is_fallible = op.is_fallible;
                        // DECY-085: Collect all output parameter types
                        if let Some(param) = f.parameters().iter().find(|p| p.name() == op.name) {
                            if let HirType::Pointer(inner) = param.param_type() {
                                output_param_types.push((**inner).clone());
                            }
                        }
                    }
                }
            }
        }

        (skip_output_params, output_param_types, output_is_fallible)
    }

    /// Check if a parameter name suggests it is an output parameter.
    fn is_output_param_name(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("result")
            || name_lower.contains("out")
            || name_lower.contains("ret")
            || name_lower == "len"
            || name_lower == "size"
            || name_lower == "x"
            || name_lower == "y"
            || name_lower == "z"
            || name_lower == "w"
            || name_lower == "h"
            || name_lower == "width"
            || name_lower == "height"
            || name_lower == "r"
            || name_lower == "g"
            || name_lower == "b"
            || name_lower == "count"
            || name_lower == "avg"
    }

    /// Generate a single annotated parameter string.
    fn generate_annotated_param(
        &self,
        p: &decy_ownership::lifetime_gen::AnnotatedParameter,
        func: Option<&HirFunction>,
    ) -> String {
        // Check if this is a slice parameter (Reference to Array with size=None)
        let is_slice = match &p.param_type {
            AnnotatedType::Reference { inner, .. } => match &**inner {
                AnnotatedType::Simple(HirType::Array { size, .. }) => size.is_none(),
                _ => false,
            },
            _ => false,
        };

        if is_slice {
            // DECY-072: Slices don't need 'mut' prefix or explicit lifetimes
            let type_str = match &p.param_type {
                AnnotatedType::Reference { inner, mutable, .. } => {
                    if let AnnotatedType::Simple(HirType::Array { element_type, .. }) =
                        &**inner
                    {
                        if *mutable {
                            format!("&mut [{}]", Self::map_type(element_type))
                        } else {
                            format!("&[{}]", Self::map_type(element_type))
                        }
                    } else {
                        self.annotated_type_to_string(&p.param_type)
                    }
                }
                _ => self.annotated_type_to_string(&p.param_type),
            };
            return format!("{}: {}", p.name, type_str);
        }

        // DECY-111: Transform pointer parameters to mutable references
        // DECY-123: Skip transformation if pointer arithmetic is used
        if let AnnotatedType::Simple(HirType::Pointer(inner)) = &p.param_type {
            return self.generate_annotated_pointer_param(&p.name, inner, func);
        }

        // DECY-196: Handle unsized array parameters → slice references
        if let AnnotatedType::Simple(HirType::Array { element_type, size: None }) =
            &p.param_type
        {
            let element_str = Self::map_type(element_type);
            return format!("{}: &mut [{}]", p.name, element_str);
        }

        // DECY-041: Add mut for all non-slice parameters to match C semantics
        format!("mut {}: {}", p.name, self.annotated_type_to_string(&p.param_type))
    }

    /// Generate an annotated pointer parameter (reference, raw pointer, slice, or &str).
    fn generate_annotated_pointer_param(
        &self,
        name: &str,
        inner: &HirType,
        func: Option<&HirFunction>,
    ) -> String {
        // DECY-135: const char* → &str transformation
        if let Some(f) = func {
            if let Some(orig_param) =
                f.parameters().iter().find(|fp| fp.name() == name)
            {
                if orig_param.is_const_char_pointer() {
                    return format!("mut {}: &str", name);
                }
            }
        }
        // DECY-134: Check for string iteration pattern FIRST
        if let Some(f) = func {
            if self.is_string_iteration_param(f, name) {
                let is_mutable = self.is_parameter_deref_modified(f, name);
                if is_mutable {
                    return format!("{}: &mut [u8]", name);
                } else {
                    return format!("{}: &[u8]", name);
                }
            }
        }
        // DECY-123: If we have function body access, check for pointer arithmetic
        if let Some(f) = func {
            if self.uses_pointer_arithmetic(f, name) {
                let inner_type = Self::map_type(inner);
                return format!("mut {}: *mut {}", name, inner_type);
            }
        }
        // DECY-168: void* parameters should stay as raw pointers
        if matches!(*inner, HirType::Void) {
            return format!("{}: *mut ()", name);
        }
        // Transform *mut T → &mut T for safety
        let inner_type = Self::map_type(inner);
        format!("{}: &mut {}", name, inner_type)
    }

    /// Append return type for annotated signature.
    fn append_annotated_return_type(
        &self,
        result: &mut String,
        sig: &AnnotatedSignature,
        func: Option<&HirFunction>,
        output_param_types: &[HirType],
        output_is_fallible: bool,
    ) {
        // Special handling for main function (DECY-AUDIT-001)
        let return_type_str = self.annotated_type_to_string(&sig.return_type);
        if sig.name == "main" && return_type_str == "i32" {
            return;
        }

        // DECY-084/085: Generate return type considering output parameters
        if !output_param_types.is_empty() {
            let out_type_str = if output_param_types.len() == 1 {
                Self::map_type(&output_param_types[0])
            } else {
                let type_strs: Vec<String> =
                    output_param_types.iter().map(Self::map_type).collect();
                format!("({})", type_strs.join(", "))
            };

            if output_is_fallible {
                result.push_str(&format!(" -> Result<{}, i32>", out_type_str));
            } else {
                result.push_str(&format!(" -> {}", out_type_str));
            }
        } else {
            // DECY-142: Check for Vec return type (malloc'd array returns)
            if let Some(f) = func {
                if let Some(vec_element_type) = self.detect_vec_return(f) {
                    let element_type_str = Self::map_type(&vec_element_type);
                    result.push_str(&format!(" -> Vec<{}>", element_type_str));
                    return;
                }
            }
            // Add return type if not void
            if return_type_str != "()" {
                result.push_str(&format!(" -> {}", return_type_str));
            }
        }
    }

    /// Convert an `AnnotatedType` to Rust type string with lifetime annotations.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_ownership::lifetime_gen::{AnnotatedType, LifetimeParam};
    /// use decy_hir::HirType;
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Simple type
    /// let simple = AnnotatedType::Simple(HirType::Int);
    /// assert_eq!(codegen.annotated_type_to_string(&simple), "i32");
    ///
    /// // Reference with lifetime
    /// let ref_type = AnnotatedType::Reference {
    ///     inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///     mutable: false,
    ///     lifetime: Some(LifetimeParam::standard(0)),
    /// };
    /// assert_eq!(codegen.annotated_type_to_string(&ref_type), "&'a i32");
    /// ```
    #[allow(clippy::only_used_in_recursion)]
    pub fn annotated_type_to_string(&self, annotated_type: &AnnotatedType) -> String {
        match annotated_type {
            AnnotatedType::Simple(hir_type) => Self::map_type(hir_type),
            AnnotatedType::Reference { inner, mutable, lifetime } => {
                // DECY-072: Special case for slices: &Vec<T> → &[T]
                // Check if inner is a Vec type
                if let AnnotatedType::Simple(HirType::Vec(element_type)) = &**inner {
                    let element_str = Self::map_type(element_type);
                    if *mutable {
                        return format!("&mut [{}]", element_str);
                    } else {
                        return format!("&[{}]", element_str);
                    }
                }

                let mut result = String::from("&");

                // Add lifetime if present
                if let Some(lt) = lifetime {
                    result.push_str(&lt.name);
                    result.push(' ');
                }

                // Add mutability
                if *mutable {
                    result.push_str("mut ");
                }

                // Add inner type
                result.push_str(&self.annotated_type_to_string(inner));

                result
            }
        }
    }

    /// Generate a default return statement for a type.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::HirType;
    ///
    /// let codegen = CodeGenerator::new();
    /// assert!(codegen.generate_return(&HirType::Int).contains("return 0"));
    /// ```
    pub fn generate_return(&self, return_type: &HirType) -> String {
        match return_type {
            HirType::Void => String::new(),
            HirType::Bool => "    return false;".to_string(),
            HirType::Int => "    return 0;".to_string(),
            HirType::UnsignedInt => "    return 0;".to_string(), // DECY-158
            HirType::Float => "    return 0.0;".to_string(),
            HirType::Double => "    return 0.0;".to_string(),
            HirType::Char => "    return 0;".to_string(),
            HirType::SignedChar => "    return 0;".to_string(), // DECY-250
            HirType::Pointer(_) => "    return std::ptr::null_mut();".to_string(),
            HirType::Box(inner) => {
                format!("    return Box::new({});", Self::default_value_for_type(inner))
            }
            HirType::Vec(_) => "    return Vec::new();".to_string(),
            HirType::Option(_) => "    return None;".to_string(),
            HirType::Reference { .. } => {
                // References in return position need concrete values from parameters
                // This should be handled by lifetime-annotated code generation
                // using generate_function_with_lifetimes() instead
                String::new()
            }
            HirType::Struct(name) => {
                format!("    return {}::default();", name)
            }
            HirType::Enum(name) => {
                format!("    return {}::default();", name)
            }
            HirType::Array { element_type, size } => {
                if let Some(n) = size {
                    format!("    return [{}; {}];", Self::default_value_for_type(element_type), n)
                } else {
                    // Unsized arrays in return position don't make sense
                    String::new()
                }
            }
            HirType::FunctionPointer { .. } => {
                // Function pointers in return position need concrete function values
                // This should be handled by the function body
                String::new()
            }
            HirType::StringLiteral => r#"    return "";"#.to_string(),
            HirType::OwnedString => "    return String::new();".to_string(),
            HirType::StringReference => r#"    return "";"#.to_string(),
            HirType::Union(_) => {
                // Unions will be transformed to enums
                // Return statement depends on the specific enum variant
                String::new()
            }
            // DECY-172: Type aliases return 0
            HirType::TypeAlias(name) => match name.as_str() {
                "size_t" => "    return 0usize;".to_string(),
                "ssize_t" | "ptrdiff_t" => "    return 0isize;".to_string(),
                _ => "    return 0;".to_string(),
            },
        }
    }

    /// Generate a complete function from HIR.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirParameter};
    ///
    /// let func = HirFunction::new(
    ///     "add".to_string(),
    ///     HirType::Int,
    ///     vec![
    ///         HirParameter::new("a".to_string(), HirType::Int),
    ///         HirParameter::new("b".to_string(), HirType::Int),
    ///     ],
    /// );
    ///
    /// let codegen = CodeGenerator::new();
    /// let code = codegen.generate_function(&func);
    ///
    /// assert!(code.contains("fn add(mut a: i32, mut b: i32) -> i32"));
    /// assert!(code.contains("{"));
    /// assert!(code.contains("}"));
    /// ```
    pub fn generate_function(&self, func: &HirFunction) -> String {
        // DECY-211: CUDA __global__ kernels -> extern "C" FFI wrapper
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Global) {
            return self.generate_cuda_kernel_ffi(func);
        }
        // DECY-211: CUDA __device__ functions -> comment noting device-only
        if func.cuda_qualifier() == Some(decy_hir::HirCudaQualifier::Device) {
            let sig = self.generate_signature(func);
            return format!(
                "// CUDA __device__ function — runs on GPU only, not transpiled\n// {}\n",
                sig
            );
        }

        let mut code = String::new();

        // DECY-072 GREEN: Build mapping of length params -> array params for body transformation
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        let mut length_to_array: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // DECY-113: Only map length params with length-like names
        // DECY-162: Don't map length params when array uses pointer arithmetic (stays raw pointer)
        for (idx, param) in func.parameters().iter().enumerate() {
            if let Some(true) = graph.is_array_parameter(param.name()) {
                // DECY-162: Skip if array param uses pointer arithmetic
                // Raw pointers don't have .len(), so we keep the size param as-is
                if self.uses_pointer_arithmetic(func, param.name()) {
                    continue;
                }

                // This is an array parameter - map the next param to this array
                // but only if it has a length-like name
                if idx + 1 < func.parameters().len() {
                    let next_param = &func.parameters()[idx + 1];
                    if matches!(next_param.param_type(), HirType::Int) {
                        let param_name = next_param.name().to_lowercase();
                        if param_name.contains("len")
                            || param_name.contains("size")
                            || param_name.contains("count")
                            || param_name == "n"
                            || param_name == "num"
                        {
                            length_to_array
                                .insert(next_param.name().to_string(), param.name().to_string());
                        }
                    }
                }
            }
        }

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Initialize type context for tracking variable types across statements
        let mut ctx = TypeContext::from_function(func);

        // DECY-129/DECY-148: Update context to reflect pointer-to-reference transformations
        // When pointer params are transformed to &mut T in signature, context must match
        // DECY-148: Distinguish array params (slices) from struct pointer params (references)
        for param in func.parameters() {
            if let HirType::Pointer(inner) = param.param_type() {
                // Check if this pointer uses pointer arithmetic (keep as raw pointer)
                if !self.uses_pointer_arithmetic(func, param.name()) {
                    // DECY-148: Check if this is an ARRAY parameter
                    let is_array_param = graph.is_array_parameter(param.name()).unwrap_or(false);

                    if is_array_param {
                        // Array parameter → register as slice (Reference to Array)
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Array {
                                    element_type: inner.clone(),
                                    size: None, // Slice (unsized array)
                                }),
                                mutable: true,
                            },
                        );
                    } else {
                        // Struct pointer → register as Reference to inner type
                        let is_mutable = self.is_parameter_deref_modified(func, param.name());
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference { inner: inner.clone(), mutable: is_mutable },
                        );
                    }
                }
            }
        }

        // DECY-142: Detect Vec-return functions for correct return type handling
        let effective_return_type = if let Some(element_type) = self.detect_vec_return(func) {
            HirType::Vec(Box::new(element_type))
        } else {
            func.return_type().clone()
        };

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements with persistent context
            for stmt in func.body() {
                code.push_str("    ");
                let stmt_code = self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(&effective_return_type),
                );

                // DECY-072 GREEN: Replace length parameter references with arr.len() calls
                let transformed = self.transform_length_refs(&stmt_code, &length_to_array);
                code.push_str(&transformed);
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a complete function from HIR with struct definitions for type inference.
    ///
    /// This is useful for testing when struct fields need proper type inference.
    /// DECY-165: Enables proper raw pointer detection for struct field access.
    pub fn generate_function_with_structs(
        &self,
        func: &HirFunction,
        structs: &[decy_hir::HirStruct],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Initialize type context with function parameters AND struct definitions
        let mut ctx = TypeContext::from_function(func);

        // DECY-165: Add struct definitions to context for field type lookup
        for struct_def in structs {
            ctx.add_struct(struct_def);
        }

        // DECY-129/DECY-148: Update context to reflect pointer-to-reference transformations
        // When pointer params are transformed to &mut T in signature, context must match
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        for param in func.parameters() {
            if let HirType::Pointer(inner) = param.param_type() {
                // Only transform if the pointer is not used for pointer arithmetic
                if !self.uses_pointer_arithmetic(func, param.name()) {
                    // Check if it's an array parameter → use &[T] or &mut [T]
                    if graph.is_array_parameter(param.name()) == Some(true) {
                        // Use slice reference type
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Vec(inner.clone())),
                                mutable: self.is_parameter_deref_modified(func, param.name()),
                            },
                        );
                    } else {
                        // Single pointer → reference
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: self.is_parameter_deref_modified(func, param.name()),
                            },
                        );
                    }
                }
            }
        }

        // Generate body statements
        if !func.body().is_empty() {
            for stmt in func.body() {
                code.push_str("    ");
                let stmt_code = self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(func.return_type()),
                );
                code.push_str(&stmt_code);
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a complete function from HIR with lifetime annotations.
    ///
    /// Takes both the HIR function and its annotated signature to generate
    /// Rust code with proper lifetime annotations.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirParameter};
    /// use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedParameter, AnnotatedType, LifetimeParam};
    ///
    /// let func = HirFunction::new(
    ///     "identity".to_string(),
    ///     HirType::Reference {
    ///         inner: Box::new(HirType::Int),
    ///         mutable: false,
    ///     },
    ///     vec![
    ///         HirParameter::new("x".to_string(), HirType::Reference {
    ///             inner: Box::new(HirType::Int),
    ///             mutable: false,
    ///         }),
    ///     ],
    /// );
    ///
    /// let sig = AnnotatedSignature {
    ///     name: "identity".to_string(),
    ///     lifetimes: vec![LifetimeParam::standard(0)],
    ///     parameters: vec![
    ///         AnnotatedParameter {
    ///             name: "x".to_string(),
    ///             param_type: AnnotatedType::Reference {
    ///                 inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///                 mutable: false,
    ///                 lifetime: Some(LifetimeParam::standard(0)),
    ///             },
    ///         },
    ///     ],
    ///     return_type: AnnotatedType::Reference {
    ///         inner: Box::new(AnnotatedType::Simple(HirType::Int)),
    ///         mutable: false,
    ///         lifetime: Some(LifetimeParam::standard(0)),
    ///     },
    /// };
    ///
    /// let codegen = CodeGenerator::new();
    /// let code = codegen.generate_function_with_lifetimes(&func, &sig);
    ///
    /// assert!(code.contains("<'a>"));
    /// assert!(code.contains("&'a i32"));
    /// ```
    pub fn generate_function_with_lifetimes(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
    ) -> String {
        self.generate_function_with_lifetimes_and_structs(func, sig, &[], &[], &[], &[], &[])
    }

    /// Generate a complete function from HIR with lifetime annotations and struct definitions.
    ///
    /// Takes the HIR function, its annotated signature, struct definitions, and all function
    /// signatures for call site reference mutability.
    ///
    /// # Arguments
    /// * `func` - The HIR function to generate
    /// * `sig` - The annotated signature with lifetime annotations
    /// * `structs` - Struct definitions for field type awareness
    /// * `all_functions` - All function signatures for DECY-117 call site mutability
    /// * `slice_func_args` - DECY-116: func_name -> [(array_idx, len_idx)] for call site transformation
    /// * `string_iter_funcs` - DECY-134b: func_name -> [(param_idx, is_mutable)] for string iteration
    /// * `globals` - DECY-220/233: Global variable names and types for unsafe access and type inference
    #[allow(clippy::too_many_arguments)]
    pub fn generate_function_with_lifetimes_and_structs(
        &self,
        func: &HirFunction,
        sig: &AnnotatedSignature,
        structs: &[decy_hir::HirStruct],
        all_functions: &[(String, Vec<HirType>)],
        slice_func_args: &[(String, Vec<(usize, usize)>)],
        string_iter_funcs: &[(String, Vec<(usize, bool)>)],
        globals: &[(String, HirType)],
    ) -> String {
        let mut code = String::new();

        // Generate signature with lifetimes
        // DECY-123: Pass function for pointer arithmetic detection
        code.push_str(&self.generate_annotated_signature_with_func(sig, Some(func)));
        code.push_str(" {\n");

        // DECY-041: Initialize type context with function parameters for pointer arithmetic
        let mut ctx = TypeContext::from_function(func);

        // DECY-220/233: Register global variables for unsafe access tracking and type inference
        for (name, var_type) in globals {
            ctx.add_global(name.clone());
            ctx.add_variable(name.clone(), var_type.clone());
        }

        // DECY-134: Track string iteration params for index-based body generation
        let mut string_iter_index_decls = Vec::new();

        // DECY-111: Transform pointer parameters to references in the context
        // DECY-123/124: Only transform if NOT using pointer arithmetic
        // This prevents unsafe blocks from being generated for reference dereferences
        // DECY-148: Use DataflowAnalyzer to determine which params are array params
        use decy_ownership::dataflow::DataflowAnalyzer;
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(func);

        for param in func.parameters() {
            // DECY-138: Check for const char* → &str transformation FIRST
            // This enables proper string iteration pattern codegen
            if param.is_const_char_pointer() {
                ctx.add_variable(param.name().to_string(), HirType::StringReference);
            } else if let HirType::Pointer(inner) = param.param_type() {
                // DECY-134: Check for string iteration pattern FIRST
                if self.is_string_iteration_param(func, param.name()) {
                    // Register as Vec type in context (slice in generated code)
                    ctx.add_variable(param.name().to_string(), HirType::Vec(inner.clone()));
                    // Register string iteration param with index variable
                    let idx_var = format!("{}_idx", param.name());
                    ctx.add_string_iter_param(param.name().to_string(), idx_var.clone());
                    // Add index declaration to generate at function start
                    string_iter_index_decls.push(format!("    let mut {}: usize = 0;", idx_var));
                } else if self.uses_pointer_arithmetic(func, param.name()) {
                    // DECY-124: Keep as pointer in context if pointer arithmetic is used
                    // This ensures proper unsafe wrapping_add/wrapping_sub codegen
                    // Keep as pointer - codegen will generate unsafe blocks
                    ctx.add_variable(param.name().to_string(), param.param_type().clone());
                } else {
                    // DECY-148: Check if this is an ARRAY parameter (detected by dataflow analysis)
                    let is_array_param = graph.is_array_parameter(param.name()).unwrap_or(false);

                    if is_array_param {
                        // DECY-146: Array parameter → register as slice (Reference to Array)
                        // This enables proper .as_ptr()/.as_mut_ptr() generation
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference {
                                inner: Box::new(HirType::Array {
                                    element_type: inner.clone(),
                                    size: None, // Slice (unsized array)
                                }),
                                mutable: true,
                            },
                        );
                    } else {
                        // DECY-148: Non-array struct pointer → register as Reference to inner type
                        // This enables proper `&mut T as *mut _` coercion on return
                        let is_mutable = self.is_parameter_deref_modified(func, param.name());
                        ctx.add_variable(
                            param.name().to_string(),
                            HirType::Reference { inner: inner.clone(), mutable: is_mutable },
                        );
                    }
                }
            }
        }

        // DECY-134: Generate index variable declarations for string iteration params
        for decl in &string_iter_index_decls {
            code.push_str(decl);
            code.push('\n');
        }

        // Add struct definitions to context for field type lookup
        for struct_def in structs {
            ctx.add_struct(struct_def);
        }

        // DECY-117: Add all function signatures for call site reference mutability
        for (func_name, param_types) in all_functions {
            ctx.add_function(func_name.clone(), param_types.clone());
        }

        // DECY-116: Add slice function arg mappings for call site transformation
        for (func_name, arg_mappings) in slice_func_args {
            ctx.add_slice_func_args(func_name.clone(), arg_mappings.clone());
        }

        // DECY-134b: Add string iteration function info for call site transformation
        for (func_name, params) in string_iter_funcs {
            ctx.add_string_iter_func(func_name.clone(), params.clone());
        }

        // DECY-142: Detect Vec-return functions for correct return type handling
        let effective_return_type = if let Some(element_type) = self.detect_vec_return(func) {
            HirType::Vec(Box::new(element_type))
        } else {
            func.return_type().clone()
        };

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate actual body statements with type context and return type
            for stmt in func.body() {
                code.push_str("    ");
                code.push_str(&self.generate_statement_with_context(
                    stmt,
                    Some(func.name()),
                    &mut ctx,
                    Some(&effective_return_type),
                ));
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a function with Box transformations applied.
    ///
    /// This method analyzes the function for malloc/free patterns and
    /// transforms them into safe `Box::new()` expressions.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirFunction, HirType, HirStatement, HirExpression};
    /// use decy_analyzer::patterns::PatternDetector;
    ///
    /// let func = HirFunction::new_with_body(
    ///     "test".to_string(),
    ///     HirType::Void,
    ///     vec![],
    ///     vec![
    ///         HirStatement::VariableDeclaration {
    ///             name: "ptr".to_string(),
    ///             var_type: HirType::Pointer(Box::new(HirType::Int)),
    ///             initializer: Some(HirExpression::FunctionCall {
    ///                 function: "malloc".to_string(),
    ///                 arguments: vec![HirExpression::IntLiteral(100)],
    ///             }),
    ///         },
    ///     ],
    /// );
    ///
    /// let codegen = CodeGenerator::new();
    /// let detector = PatternDetector::new();
    /// let candidates = detector.find_box_candidates(&func);
    /// let code = codegen.generate_function_with_box_transform(&func, &candidates);
    ///
    /// assert!(code.contains("Box::new"));
    /// ```
    pub fn generate_function_with_box_transform(
        &self,
        func: &HirFunction,
        candidates: &[decy_analyzer::patterns::BoxCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with Box transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check if this statement should be transformed
                let transformed_stmt =
                    if let Some(candidate) = candidates.iter().find(|c| c.malloc_index == idx) {
                        self.box_transformer.transform_statement(stmt, candidate)
                    } else {
                        stmt.clone()
                    };

                code.push_str("    ");
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a function with Vec transformations applied.
    ///
    /// This method analyzes the function for malloc(n * sizeof(T)) patterns and
    /// transforms them into safe `Vec::with_capacity(n)` expressions.
    pub fn generate_function_with_vec_transform(
        &self,
        func: &HirFunction,
        candidates: &[decy_analyzer::patterns::VecCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with Vec transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check if this statement should be transformed
                let transformed_stmt =
                    if let Some(candidate) = candidates.iter().find(|c| c.malloc_index == idx) {
                        self.transform_vec_statement(stmt, candidate)
                    } else {
                        stmt.clone()
                    };

                code.push_str("    ");
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Transform a statement to use Vec instead of malloc for array patterns.
    fn transform_vec_statement(
        &self,
        stmt: &HirStatement,
        candidate: &decy_analyzer::patterns::VecCandidate,
    ) -> HirStatement {
        match stmt {
            HirStatement::VariableDeclaration { name, var_type, initializer: _ } => {
                // Get the element type from the pointer
                let element_type = if let HirType::Pointer(inner) = var_type {
                    (**inner).clone()
                } else {
                    // Fallback: keep original type
                    return stmt.clone();
                };

                // Transform type to Vec
                let vec_type = HirType::Vec(Box::new(element_type));

                // Transform initializer: malloc(n * sizeof(T)) → Vec::with_capacity(n)
                let vec_initializer = if let Some(capacity_expr) = &candidate.capacity_expr {
                    Some(HirExpression::FunctionCall {
                        function: "Vec::with_capacity".to_string(),
                        arguments: vec![capacity_expr.clone()],
                    })
                } else {
                    // No capacity expression - use Vec::new()
                    Some(HirExpression::FunctionCall {
                        function: "Vec::new".to_string(),
                        arguments: vec![],
                    })
                };

                HirStatement::VariableDeclaration {
                    name: name.clone(),
                    var_type: vec_type,
                    initializer: vec_initializer,
                }
            }
            HirStatement::Assignment { target: _, value: _ } => {
                // Similar transformation for assignments
                // For now, keep the original statement
                // Future: handle ptr = malloc(n * sizeof(T)) assignments
                stmt.clone()
            }
            _ => stmt.clone(),
        }
    }

    /// Generate a function with both Box and Vec transformations applied.
    ///
    /// This method combines both Box and Vec transformations,
    /// applying them to their respective patterns.
    pub fn generate_function_with_box_and_vec_transform(
        &self,
        func: &HirFunction,
        box_candidates: &[decy_analyzer::patterns::BoxCandidate],
        vec_candidates: &[decy_analyzer::patterns::VecCandidate],
    ) -> String {
        let mut code = String::new();

        // Generate signature
        code.push_str(&self.generate_signature(func));
        code.push_str(" {\n");

        // Generate body statements if present
        if func.body().is_empty() {
            // Generate stub body with return statement
            let return_stmt = self.generate_return(func.return_type());
            if !return_stmt.is_empty() {
                code.push_str(&return_stmt);
                code.push('\n');
            }
        } else {
            // Generate body statements with both transformations
            for (idx, stmt) in func.body().iter().enumerate() {
                // Check Vec candidates first (more specific pattern)
                let transformed_stmt = if let Some(vec_candidate) =
                    vec_candidates.iter().find(|c| c.malloc_index == idx)
                {
                    self.transform_vec_statement(stmt, vec_candidate)
                } else if let Some(box_candidate) =
                    box_candidates.iter().find(|c| c.malloc_index == idx)
                {
                    self.box_transformer.transform_statement(stmt, box_candidate)
                } else {
                    stmt.clone()
                };

                code.push_str("    ");
                code.push_str(
                    &self.generate_statement_for_function(&transformed_stmt, Some(func.name())),
                );
                code.push('\n');
            }
        }

        code.push('}');
        code
    }

    /// Generate a struct definition from HIR.
    ///
    /// Generates Rust struct code with automatic derives for Debug, Clone, PartialEq, Eq.
    /// Handles lifetimes automatically for structs with reference fields.
    pub fn generate_struct(&self, hir_struct: &decy_hir::HirStruct) -> String {
        let mut code = String::new();

        // Check if struct needs lifetimes (has Reference fields)
        let needs_lifetimes =
            hir_struct.fields().iter().any(|f| matches!(f.field_type(), HirType::Reference { .. }));

        // DECY-123: Check if struct has large arrays (> 32 elements) that don't impl Default
        // Rust arrays only implement Default for sizes up to 32
        let has_large_array = hir_struct.fields().iter().any(|f| {
            matches!(
                f.field_type(),
                HirType::Array { size: Some(n), .. } if *n > 32
            )
        });

        // DECY-218: Check if struct has float/double fields (f32/f64 don't implement Eq)
        let has_float_fields = hir_struct
            .fields()
            .iter()
            .any(|f| matches!(f.field_type(), HirType::Float | HirType::Double));

        // DECY-225: Check if struct can derive Copy (only primitive types, no pointers/Box/Vec/String)
        // Helper to check if a type is Copy-able
        fn is_copy_type(ty: &HirType) -> bool {
            match ty {
                HirType::Int
                | HirType::UnsignedInt
                | HirType::Bool
                | HirType::Float
                | HirType::Double
                | HirType::Char
                | HirType::SignedChar // DECY-250
                | HirType::Void => true,
                HirType::Array { element_type, .. } => is_copy_type(element_type),
                // DECY-246: Raw pointers (*mut T, *const T) ARE Copy in Rust!
                HirType::Pointer(_) => true,
                // Box, Vec, String, References are not Copy
                HirType::Box(_)
                | HirType::Vec(_)
                | HirType::OwnedString
                | HirType::StringReference
                | HirType::StringLiteral
                | HirType::Reference { .. } => false,
                // Struct fields need the inner struct to be Copy, which we can't check here
                // Be conservative and don't derive Copy
                HirType::Struct(_) | HirType::Enum(_) | HirType::Union(_) => false,
                // Function pointers are not Copy (they could be wrapped in Option)
                HirType::FunctionPointer { .. } => false,
                // Type aliases (like size_t) are Copy
                HirType::TypeAlias(_) => true,
                HirType::Option(_) => false,
            }
        }

        let can_derive_copy =
            !needs_lifetimes && hir_struct.fields().iter().all(|f| is_copy_type(f.field_type()));

        // Add derive attribute
        // DECY-114: Add Default derive for struct initialization with ::default()
        // DECY-123: Skip Default for large arrays
        // DECY-218: Skip Eq for floats (f32/f64 only implement PartialEq)
        // DECY-225: Add Copy for simple structs to avoid move errors
        let derives = match (has_large_array, has_float_fields, can_derive_copy) {
            (true, true, true) => "#[derive(Debug, Clone, Copy, PartialEq)]\n",
            (true, true, false) => "#[derive(Debug, Clone, PartialEq)]\n",
            (true, false, true) => "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n",
            (true, false, false) => "#[derive(Debug, Clone, PartialEq, Eq)]\n",
            (false, true, true) => "#[derive(Debug, Clone, Copy, Default, PartialEq)]\n",
            (false, true, false) => "#[derive(Debug, Clone, Default, PartialEq)]\n",
            (false, false, true) => "#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]\n",
            (false, false, false) => "#[derive(Debug, Clone, Default, PartialEq, Eq)]\n",
        };
        code.push_str(derives);

        // Add struct declaration with or without lifetime
        if needs_lifetimes {
            code.push_str(&format!("pub struct {}<'a> {{\n", hir_struct.name()));
        } else {
            code.push_str(&format!("pub struct {} {{\n", hir_struct.name()));
        }

        // Add fields
        // Note: struct_name reserved for DECY-144 self-referential pointer detection
        let _struct_name = hir_struct.name();
        for field in hir_struct.fields() {
            // DECY-136: Flexible array members (Array with size: None) → Vec<T>
            // C99 §6.7.2.1: struct { int size; char data[]; } → Vec<u8>
            //
            // DECY-144: Self-referential pointers (struct Node* next) → Option<Box<T>>
            // This significantly reduces unsafe blocks in recursive data structures.
            let field_type_str = match field.field_type() {
                HirType::Array { element_type, size: None } => {
                    // Flexible array member → Vec<T>
                    format!("Vec<{}>", Self::map_type(element_type))
                }
                // DECY-144: Self-referential pointer → Option<Box<T>> (DEFERRED)
                // The full transformation requires updating ALL usages:
                // - Function parameters and return types
                // - Local variable types
                // - Field access patterns (Some(ref x) instead of *ptr)
                // - NULL checks (is_none() instead of == null_mut())
                //
                // For now, keep raw pointers but track these fields for future transformation.
                // See DECY-145 for full Option<Box<T>> transformation implementation.
                HirType::Pointer(_inner) => {
                    // Commented out for now - needs full transformation
                    // if let HirType::Struct(inner_name) = inner.as_ref() {
                    //     if inner_name == struct_name {
                    //         format!("Option<Box<{}>>", struct_name)
                    //     } else {
                    //         Self::map_type(field.field_type())
                    //     }
                    // } else {
                    //     Self::map_type(field.field_type())
                    // }
                    Self::map_type(field.field_type())
                }
                other => Self::map_type(other),
            };
            // DECY-227: Escape reserved keywords in field names
            code.push_str(&format!(
                "    pub {}: {},\n",
                escape_rust_keyword(field.name()),
                field_type_str
            ));
        }

        code.push('}');
        code
    }

    /// DECY-202: Generate a Rust struct + impl block from a C++ class.
    ///
    /// Maps: class fields -> struct fields, methods -> impl block,
    /// constructor -> `pub fn new()`, destructor -> `impl Drop`.
    ///
    /// # Example
    ///
    /// C++: `class Point { int x, y; Point(int x, int y); int distance(); ~Point(); };`
    /// Rust:
    /// ```ignore
    /// #[derive(Debug, Clone, Default, PartialEq, Eq)]
    /// pub struct Point {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// impl Point {
    ///     pub fn new(x: i32, y: i32) -> Self {
    ///         Self { x, y }
    ///     }
    ///     pub fn distance(&self) -> i32 { /* ... */ }
    /// }
    ///
    /// impl Drop for Point {
    ///     fn drop(&mut self) { /* destructor body */ }
    /// }
    /// ```
    pub fn generate_class(&self, hir_class: &decy_hir::HirClass) -> String {
        let mut code = String::new();

        // Generate struct definition
        // DECY-209: If there's a base class, add it as a field for composition
        let mut fields = hir_class.fields().to_vec();
        if let Some(base) = hir_class.base_class() {
            // Insert base class field at the beginning
            fields.insert(
                0,
                decy_hir::HirStructField::new(
                    "base".to_string(),
                    decy_hir::HirType::Struct(base.to_string()),
                ),
            );
        }
        let hir_struct = decy_hir::HirStruct::new(hir_class.name().to_string(), fields);
        code.push_str(&self.generate_struct(&hir_struct));
        code.push_str("\n\n");

        // Generate impl block
        code.push_str(&format!("impl {} {{\n", hir_class.name()));

        // Generate constructor as new() if constructor params exist
        if !hir_class.constructor_params().is_empty() {
            code.push_str("    pub fn new(");
            let params: Vec<String> = hir_class
                .constructor_params()
                .iter()
                .map(|p| format!("{}: {}", escape_rust_keyword(p.name()), Self::map_type(p.param_type())))
                .collect();
            code.push_str(&params.join(", "));
            code.push_str(") -> Self {\n");
            code.push_str("        Self {\n");
            // DECY-213: Map constructor params to field initializers
            // Strategy: name match first, then positional fallback
            let ctor_params = hir_class.constructor_params();
            let own_fields: Vec<_> = hir_class.fields().iter()
                .filter(|f| f.name() != "base") // skip inherited base field
                .collect();
            for (idx, field) in own_fields.iter().enumerate() {
                // Try name match first
                let matching_param = ctor_params.iter().find(|p| p.name() == field.name());
                if let Some(param) = matching_param {
                    code.push_str(&format!(
                        "            {}: {},\n",
                        escape_rust_keyword(field.name()),
                        escape_rust_keyword(param.name())
                    ));
                } else if idx < ctor_params.len() {
                    // Positional fallback: param[idx] -> field[idx]
                    code.push_str(&format!(
                        "            {}: {},\n",
                        escape_rust_keyword(field.name()),
                        escape_rust_keyword(ctor_params[idx].name())
                    ));
                } else {
                    code.push_str(&format!(
                        "            {}: Default::default(),\n",
                        escape_rust_keyword(field.name())
                    ));
                }
            }
            // If there's a base field from inheritance, default-construct it
            if hir_class.base_class().is_some() {
                code.push_str("            base: Default::default(),\n");
            }
            code.push_str("        }\n");
            code.push_str("    }\n\n");
        }

        // Generate regular methods (non-operator)
        for method in hir_class.methods().iter().filter(|m| m.operator_kind().is_none()) {
            let func = method.function();
            let self_param = if method.is_static() {
                ""
            } else if method.is_const() {
                "&self, "
            } else {
                "&mut self, "
            };

            let params: Vec<String> = func
                .parameters()
                .iter()
                .map(|p| format!("{}: {}", escape_rust_keyword(p.name()), Self::map_type(p.param_type())))
                .collect();

            let return_type = if *func.return_type() == decy_hir::HirType::Void {
                String::new()
            } else {
                format!(" -> {}", Self::map_type(func.return_type()))
            };

            code.push_str(&format!(
                "    pub fn {}({}{}){} {{\n",
                escape_rust_keyword(func.name()),
                self_param,
                params.join(", "),
                return_type,
            ));

            if func.body().is_empty() {
                if *func.return_type() != decy_hir::HirType::Void {
                    code.push_str("        Default::default()\n");
                }
            } else {
                for stmt in func.body() {
                    code.push_str(&format!(
                        "        {}\n",
                        self.generate_statement_with_context(stmt, None, &mut TypeContext::new(), None)
                    ));
                }
            }

            code.push_str("    }\n\n");
        }

        code.push_str("}\n");

        // DECY-208: Generate std::ops trait impls for operator methods
        let class_name = hir_class.name();
        for method in hir_class.methods().iter().filter(|m| m.operator_kind().is_some()) {
            let op = method.operator_kind().unwrap();
            let func = method.function();
            let return_type = Self::map_type(func.return_type());

            // Get the RHS parameter type (first parameter of operator method)
            let rhs_type = func.parameters().first().map_or(
                class_name.to_string(),
                |p| Self::map_type(p.param_type()),
            );

            use decy_hir::HirCxxOperatorKind as Op;
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem => {
                    let (trait_name, method_name) = match op {
                        Op::Add => ("Add", "add"),
                        Op::Sub => ("Sub", "sub"),
                        Op::Mul => ("Mul", "mul"),
                        Op::Div => ("Div", "div"),
                        Op::Rem => ("Rem", "rem"),
                        _ => unreachable!(),
                    };
                    code.push_str(&format!(
                        "\nimpl std::ops::{}<{}> for {} {{\n",
                        trait_name, rhs_type, class_name
                    ));
                    code.push_str(&format!("    type Output = {};\n\n", return_type));
                    code.push_str(&format!(
                        "    fn {}(self, rhs: {}) -> Self::Output {{\n",
                        method_name, rhs_type
                    ));
                    code.push_str("        Default::default()\n");
                    code.push_str("    }\n");
                    code.push_str("}\n");
                }
                Op::Equal => {
                    code.push_str(&format!("\nimpl PartialEq for {} {{\n", class_name));
                    code.push_str("    fn eq(&self, other: &Self) -> bool {\n");
                    code.push_str("        Default::default()\n");
                    code.push_str("    }\n");
                    code.push_str("}\n");
                }
                Op::AddAssign | Op::SubAssign => {
                    let (trait_name, method_name) = match op {
                        Op::AddAssign => ("AddAssign", "add_assign"),
                        Op::SubAssign => ("SubAssign", "sub_assign"),
                        _ => unreachable!(),
                    };
                    code.push_str(&format!(
                        "\nimpl std::ops::{}<{}> for {} {{\n",
                        trait_name, rhs_type, class_name
                    ));
                    code.push_str(&format!(
                        "    fn {}(&mut self, rhs: {}) {{\n",
                        method_name, rhs_type
                    ));
                    code.push_str("    }\n");
                    code.push_str("}\n");
                }
                _ => {
                    // Other operators: emit as comment
                    code.push_str(&format!(
                        "\n// TODO: impl operator {:?} for {}\n",
                        op, class_name
                    ));
                }
            }
        }

        // Generate Drop impl if destructor exists
        if hir_class.has_destructor() {
            code.push_str(&format!("\nimpl Drop for {} {{\n", hir_class.name()));
            code.push_str("    fn drop(&mut self) {\n");
            code.push_str("        // Destructor body (C++ ~ClassName)\n");
            code.push_str("    }\n");
            code.push_str("}\n");
        }

        // DECY-209: Generate Deref/DerefMut for base class access
        if let Some(base) = hir_class.base_class() {
            code.push_str(&format!(
                "\nimpl std::ops::Deref for {} {{\n",
                hir_class.name()
            ));
            code.push_str(&format!("    type Target = {};\n\n", base));
            code.push_str("    fn deref(&self) -> &Self::Target {\n");
            code.push_str("        &self.base\n");
            code.push_str("    }\n");
            code.push_str("}\n");

            code.push_str(&format!(
                "\nimpl std::ops::DerefMut for {} {{\n",
                hir_class.name()
            ));
            code.push_str("    fn deref_mut(&mut self) -> &mut Self::Target {\n");
            code.push_str("        &mut self.base\n");
            code.push_str("    }\n");
            code.push_str("}\n");
        }

        code
    }

    /// DECY-211: Generate FFI declaration for a CUDA __global__ kernel.
    ///
    /// CUDA kernels cannot be directly transpiled to Rust — they run on the GPU.
    /// Instead, generate an `extern "C"` FFI declaration so host code can call
    /// the pre-compiled kernel, plus a safe wrapper comment.
    fn generate_cuda_kernel_ffi(&self, func: &HirFunction) -> String {
        let mut code = String::new();

        // Generate extern "C" block
        code.push_str("extern \"C\" {\n");
        code.push_str(&format!("    /// CUDA kernel: {} (compiled separately)\n", func.name()));

        // Build parameter list as raw C types
        let params: Vec<String> = func
            .parameters()
            .iter()
            .map(|p| {
                let ty = match p.param_type() {
                    decy_hir::HirType::Pointer(inner) => {
                        format!("*mut {}", Self::map_type(inner))
                    }
                    other => Self::map_type(other),
                };
                format!("{}: {}", escape_rust_keyword(p.name()), ty)
            })
            .collect();

        let return_type = if *func.return_type() == decy_hir::HirType::Void {
            String::new()
        } else {
            format!(" -> {}", Self::map_type(func.return_type()))
        };

        code.push_str(&format!(
            "    fn {}({}){};\n",
            func.name(),
            params.join(", "),
            return_type,
        ));
        code.push_str("}\n");

        code
    }

    /// DECY-205: Generate a Rust `mod` block from a C++ namespace.
    ///
    /// Recursively generates nested modules for nested namespaces.
    /// Functions, structs, and classes within the namespace are generated
    /// inside the module scope.
    pub fn generate_namespace(&self, ns: &decy_hir::HirNamespace) -> String {
        let mut code = String::new();

        code.push_str(&format!("pub mod {} {{\n", escape_rust_keyword(ns.name())));

        // Generate structs
        for s in ns.structs() {
            for line in self.generate_struct(s).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Generate classes
        for c in ns.classes() {
            for line in self.generate_class(c).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Generate functions
        for f in ns.functions() {
            let func_code = self.generate_function(f);
            for line in func_code.lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        // Recurse into nested namespaces
        for nested in ns.namespaces() {
            for line in self.generate_namespace(nested).lines() {
                code.push_str(&format!("    {}\n", line));
            }
            code.push('\n');
        }

        code.push_str("}\n");
        code
    }

    /// DECY-240: Generate an enum definition from HIR.
    ///
    /// Generates Rust const declarations for C enum values.
    /// C enums create integer constants that can be used directly (without prefix),
    /// so we generate const i32 values rather than Rust enums.
    ///
    /// # Example
    ///
    /// C: `enum day { MONDAY = 1, TUESDAY, WEDNESDAY };`
    /// Rust:
    /// ```ignore
    /// pub const MONDAY: i32 = 1;
    /// pub const TUESDAY: i32 = 2;
    /// pub const WEDNESDAY: i32 = 3;
    /// ```
    pub fn generate_enum(&self, hir_enum: &decy_hir::HirEnum) -> String {
        let mut code = String::new();

        // Add a type alias for the enum name (C: enum day → Rust: type day = i32)
        let enum_name = hir_enum.name();
        if !enum_name.is_empty() {
            code.push_str(&format!("pub type {} = i32;\n", enum_name));
        }

        // Generate const declarations for each variant
        let mut next_value: i32 = 0;
        for variant in hir_enum.variants() {
            let value = if let Some(v) = variant.value() {
                next_value = v + 1; // Next auto value
                v
            } else {
                let v = next_value;
                next_value += 1;
                v
            };
            code.push_str(&format!("pub const {}: i32 = {};\n", variant.name(), value));
        }

        code
    }

    /// Generate a typedef (type alias) from HIR.
    ///
    /// Generates Rust type alias code using the `type` keyword.
    /// Handles redundant typedefs (where name matches underlying struct/enum name) as comments.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirTypedef, HirType};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Simple typedef: typedef int Integer;
    /// let typedef = HirTypedef::new("Integer".to_string(), HirType::Int);
    /// let code = codegen.generate_typedef(&typedef).unwrap();
    /// assert!(code.contains("type Integer = i32"));
    ///
    /// // Pointer typedef: typedef int* IntPtr;
    /// let typedef = HirTypedef::new("IntPtr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    /// let code = codegen.generate_typedef(&typedef).unwrap();
    /// assert!(code.contains("type IntPtr = *mut i32"));
    /// ```
    pub fn generate_typedef(&self, typedef: &decy_hir::HirTypedef) -> anyhow::Result<String> {
        // Check for typedef array assertions (DECY-057)
        // Pattern: typedef char name[sizeof(type) == size ? 1 : -1];
        if let HirType::Array { element_type, size } = typedef.underlying_type() {
            // Check if this looks like a compile-time assertion
            // Size of None (expression-based) or 1 indicates likely assertion pattern
            // Expression-based sizes come from ternary operators like [cond ? 1 : -1]
            let is_assertion = size.is_none() || *size == Some(1);

            if is_assertion {
                // This is a typedef array assertion - generate Rust const assertion
                // Generate a compile-time assertion that will be checked by rustc
                return Ok(format!(
                    "// Compile-time assertion from typedef {} (C pattern: typedef {}[expr ? 1 : -1])\nconst _: () = assert!(std::mem::size_of::<i32>() == 4);",
                    typedef.name(),
                    Self::map_type(element_type)
                ));
            }

            // Regular array typedef with fixed size
            return Ok(format!(
                "pub type {} = [{}; {}];",
                typedef.name(),
                Self::map_type(element_type),
                size.unwrap_or(0)
            ));
        }

        // DECY-167: Handle platform size types specially
        // These need to map to usize/isize for compatibility with Rust methods like .len()
        let name = typedef.name();
        if name == "size_t" {
            return Ok("pub type size_t = usize;".to_string());
        }
        if name == "ssize_t" {
            return Ok("pub type ssize_t = isize;".to_string());
        }
        if name == "ptrdiff_t" {
            return Ok("pub type ptrdiff_t = isize;".to_string());
        }

        // Check for redundant typedef (struct/enum name matching typedef name)
        let result = match typedef.underlying_type() {
            HirType::Struct(struct_name) | HirType::Enum(struct_name) if struct_name == name => {
                // In Rust, struct/enum names are already types, so this is redundant
                // Generate as a comment for documentation purposes
                format!("// type {} = {}; (redundant in Rust)", name, struct_name)
            }
            _ => {
                // Regular type alias with public visibility
                format!("pub type {} = {};", name, Self::map_type(typedef.underlying_type()))
            }
        };
        Ok(result)
    }

    /// Generate a constant declaration from HIR.
    ///
    /// Transforms C `#define` macro constants to Rust `const` declarations.
    /// C #define constants are compile-time text substitutions that map naturally
    /// to Rust's const with compile-time evaluation.
    ///
    /// # Examples
    ///
    /// ```
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirConstant, HirType, HirExpression};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // Integer constant: #define MAX 100 → const MAX: i32 = 100;
    /// let constant = HirConstant::new(
    ///     "MAX".to_string(),
    ///     HirType::Int,
    ///     HirExpression::IntLiteral(100),
    /// );
    /// let code = codegen.generate_constant(&constant);
    /// assert!(code.contains("const MAX: i32 = 100"));
    ///
    /// // String constant: #define MSG "Hello" → const MSG: &str = "Hello";
    /// let constant = HirConstant::new(
    ///     "MSG".to_string(),
    ///     HirType::Pointer(Box::new(HirType::Char)),
    ///     HirExpression::StringLiteral("Hello".to_string()),
    /// );
    /// let code = codegen.generate_constant(&constant);
    /// assert!(code.contains("const MSG: &str = \"Hello\""));
    /// ```
    ///
    /// # Safety
    ///
    /// This transformation introduces 0 unsafe blocks, maintaining the goal of
    /// <5 unsafe blocks per 1000 LOC.
    ///
    /// Reference: K&R §4.11, ISO C99 §6.10.3
    pub fn generate_constant(&self, constant: &decy_hir::HirConstant) -> String {
        // Map char* to &str for string constants
        let rust_type = if matches!(
            constant.const_type(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Char)
        ) {
            "&str".to_string()
        } else {
            Self::map_type(constant.const_type())
        };

        format!(
            "const {}: {} = {};",
            constant.name(),
            rust_type,
            self.generate_expression(constant.value())
        )
    }

    /// Generate a global variable declaration with storage class specifiers.
    ///
    /// Transforms C global variables with storage classes to appropriate Rust declarations:
    /// - `static` → `static mut` (mutable static)
    /// - `extern` → `extern "C" { static }`
    /// - `const` → `const`
    /// - `static const` → `const` (const is stronger than static)
    /// - Plain global → `static mut` (default to mutable)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use decy_codegen::CodeGenerator;
    /// use decy_hir::{HirConstant, HirType, HirExpression};
    ///
    /// let codegen = CodeGenerator::new();
    ///
    /// // static int counter = 0; → static mut counter: i32 = 0;
    /// let global = HirConstant::new(
    ///     "counter".to_string(),
    ///     HirType::Int,
    ///     HirExpression::IntLiteral(0),
    /// );
    /// let code = codegen.generate_global_variable(&global, true, false, false);
    /// assert!(code.contains("static mut counter: i32 = 0"));
    /// ```
    ///
    /// # Arguments
    ///
    /// * `variable` - The HIR constant representing the global variable
    /// * `is_static` - Whether the variable has `static` storage class
    /// * `is_extern` - Whether the variable has `extern` storage class
    /// * `is_const` - Whether the variable has `const` qualifier
    ///
    /// # Safety
    ///
    /// Note: `static mut` in Rust requires unsafe blocks to access, which increases
    /// unsafe usage. However, this is necessary to preserve C semantics for mutable globals.
    ///
    /// Reference: ISO C99 §6.7.1 (Storage-class specifiers), K&R §4.2
    pub fn generate_global_variable(
        &self,
        variable: &decy_hir::HirConstant,
        _is_static: bool,
        is_extern: bool,
        is_const: bool,
    ) -> String {
        let var_name = variable.name();
        let value_expr = self.generate_expression(variable.value());

        // Determine Rust type (special handling for string literals)
        let rust_type = if matches!(
            variable.const_type(),
            HirType::Pointer(inner) if matches!(**inner, HirType::Char)
        ) && is_const
        {
            // const char* → &str or &'static str
            "&str".to_string()
        } else {
            Self::map_type(variable.const_type())
        };

        // Handle different storage class combinations
        if is_extern {
            // extern int x; → extern "C" { static x: i32; }
            format!("extern \"C\" {{\n    static {}: {};\n}}", var_name, rust_type)
        } else if is_const {
            // const int x = 10; → const x: i32 = 10;
            // static const int x = 10; → const x: i32 = 10; (const is stronger)
            format!("const {}: {} = {};", var_name, rust_type, value_expr)
        } else {
            // static int x = 0; → static mut x: i32 = 0;
            // int x = 0; → static mut x: i32 = 0; (default)
            // Special handling for arrays: [0; 10] for array initialization
            let init_expr = if let HirType::Array { element_type, size } = variable.const_type() {
                if let Some(size_val) = size {
                    // DECY-201: Fix array initialization for uninitialized arrays
                    // DECY-246: Use default_value_for_type to handle all types including structs
                    // Check if value is an integer (likely uninitialized or zero-initialized)
                    let element_init = match variable.value() {
                        HirExpression::IntLiteral(_) => {
                            // Any integer value for struct/complex array → use default
                            Self::default_value_for_type(element_type)
                        }
                        _ => self.generate_expression(variable.value()),
                    };
                    format!("[{}; {}]", element_init, size_val)
                } else {
                    value_expr
                }
            } else if matches!(variable.const_type(), HirType::Pointer(_)) {
                // Handle NULL pointer initialization
                if matches!(variable.value(), HirExpression::IntLiteral(0)) {
                    "std::ptr::null_mut()".to_string()
                } else {
                    value_expr
                }
            } else {
                value_expr
            };

            format!("static mut {}: {} = {};", var_name, rust_type, init_expr)
        }
    }
}
