//! Concurrency transformation module for pthread → Rust std::sync conversions.
//!
//! Transforms C pthread synchronization primitives to safe Rust equivalents:
//! - pthread_mutex_t + data → `Mutex<T>`
//! - pthread_mutex_lock/unlock → `.lock().unwrap()` with RAII
//!
//! Part of DECY-078: Transform pthread_mutex to `Mutex<T>`

use decy_hir::{HirExpression, HirFunction, HirStatement};

/// Detects if a function call is a pthread mutex lock operation.
///
/// Recognizes patterns:
/// - pthread_mutex_lock(&mutex)
/// - pthread_mutex_lock(&ptr->mutex)
///
/// Returns the name of the mutex variable if detected, None otherwise.
pub fn is_pthread_lock(stmt: &HirStatement) -> Option<String> {
    if let HirStatement::Expression(HirExpression::FunctionCall {
        function,
        arguments,
    }) = stmt
    {
        if function == "pthread_mutex_lock" && !arguments.is_empty() {
            // Extract mutex name from &mutex or &ptr->field
            if let Some(HirExpression::AddressOf(inner)) = arguments.first() {
                return extract_variable_name(inner);
            }
        }
    }
    None
}

/// Detects if a function call is a pthread mutex unlock operation.
///
/// Recognizes patterns:
/// - pthread_mutex_unlock(&mutex)
/// - pthread_mutex_unlock(&ptr->mutex)
///
/// Returns the name of the mutex variable if detected, None otherwise.
pub fn is_pthread_unlock(stmt: &HirStatement) -> Option<String> {
    if let HirStatement::Expression(HirExpression::FunctionCall {
        function,
        arguments,
    }) = stmt
    {
        if function == "pthread_mutex_unlock" && !arguments.is_empty() {
            // Extract mutex name from &mutex or &ptr->field
            if let Some(HirExpression::AddressOf(inner)) = arguments.first() {
                return extract_variable_name(inner);
            }
        }
    }
    None
}

/// Extracts the variable name from an expression.
///
/// Handles:
/// - HirExpression::Variable(name) → Some(name)
/// - HirExpression::PointerFieldAccess → Some(field_name)
/// - HirExpression::FieldAccess → Some(field_name)
/// - Other expressions → None
fn extract_variable_name(expr: &HirExpression) -> Option<String> {
    match expr {
        HirExpression::Variable(name) => Some(name.clone()),
        HirExpression::PointerFieldAccess { field, .. } => Some(field.clone()),
        HirExpression::FieldAccess { field, .. } => Some(field.clone()),
        _ => None,
    }
}

/// Identifies lock regions in a function body.
///
/// A lock region is a sequence of statements between pthread_mutex_lock
/// and pthread_mutex_unlock for the same mutex.
///
/// Returns a vector of (lock_name, start_index, end_index) tuples.
pub fn identify_lock_regions(func: &HirFunction) -> Vec<(String, usize, usize)> {
    let mut regions = Vec::new();
    let body = func.body();
    let mut active_locks: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (idx, stmt) in body.iter().enumerate() {
        if let Some(lock_name) = is_pthread_lock(stmt) {
            active_locks.insert(lock_name, idx);
        } else if let Some(unlock_name) = is_pthread_unlock(stmt) {
            if let Some(start_idx) = active_locks.remove(&unlock_name) {
                regions.push((unlock_name, start_idx, idx));
            }
        }
    }

    regions
}

/// Checks if a function contains any pthread mutex operations.
pub fn has_pthread_mutex_calls(func: &HirFunction) -> bool {
    func.body()
        .iter()
        .any(|stmt| is_pthread_lock(stmt).is_some() || is_pthread_unlock(stmt).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::HirType;

    fn lock_call(lock_name: &str) -> HirStatement {
        HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_lock".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
                lock_name.to_string(),
            )))],
        })
    }

    fn unlock_call(lock_name: &str) -> HirStatement {
        HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_unlock".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
                lock_name.to_string(),
            )))],
        })
    }

    #[test]
    fn test_detect_pthread_lock_call() {
        let stmt = lock_call("my_mutex");
        assert_eq!(is_pthread_lock(&stmt), Some("my_mutex".to_string()));
    }

    #[test]
    fn test_detect_pthread_unlock_call() {
        let stmt = unlock_call("my_mutex");
        assert_eq!(is_pthread_unlock(&stmt), Some("my_mutex".to_string()));
    }

    #[test]
    fn test_non_pthread_call_not_detected() {
        let stmt = HirStatement::Expression(HirExpression::FunctionCall {
            function: "some_other_function".to_string(),
            arguments: vec![],
        });
        assert_eq!(is_pthread_lock(&stmt), None);
        assert_eq!(is_pthread_unlock(&stmt), None);
    }

    #[test]
    fn test_identify_single_lock_region() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_call("lock"),
                HirStatement::Assignment {
                    target: "data".to_string(),
                    value: HirExpression::IntLiteral(42),
                },
                unlock_call("lock"),
            ],
        );

        let regions = identify_lock_regions(&func);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].0, "lock");
        assert_eq!(regions[0].1, 0); // start index
        assert_eq!(regions[0].2, 2); // end index
    }

    #[test]
    fn test_identify_multiple_lock_regions() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_call("lock1"),
                unlock_call("lock1"),
                lock_call("lock2"),
                unlock_call("lock2"),
            ],
        );

        let regions = identify_lock_regions(&func);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].0, "lock1");
        assert_eq!(regions[1].0, "lock2");
    }

    #[test]
    fn test_has_pthread_mutex_calls() {
        let func_with_mutex = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![lock_call("lock"), unlock_call("lock")],
        );

        let func_without_mutex = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
        );

        assert!(has_pthread_mutex_calls(&func_with_mutex));
        assert!(!has_pthread_mutex_calls(&func_without_mutex));
    }

    #[test]
    fn test_extract_variable_name_from_variable() {
        let expr = HirExpression::Variable("my_var".to_string());
        assert_eq!(extract_variable_name(&expr), Some("my_var".to_string()));
    }

    #[test]
    fn test_extract_variable_name_from_field_access() {
        let expr = HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("obj".to_string())),
            field: "field_name".to_string(),
        };
        assert_eq!(extract_variable_name(&expr), Some("field_name".to_string()));
    }

    #[test]
    fn test_extract_variable_name_from_literal_returns_none() {
        let expr = HirExpression::IntLiteral(42);
        assert_eq!(extract_variable_name(&expr), None);
    }
}
