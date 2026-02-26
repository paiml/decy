//! Basic Probar test
use jugar_probar::prelude::*;

#[test]
fn test_example() {
    let result = TestResult::pass("example_test");
    assert!(result.passed);
}
