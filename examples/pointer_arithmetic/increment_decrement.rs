// DECY-041: Test increment/decrement operators
// Transpiled to idiomatic Rust

// Post-increment
fn post_increment_test() -> i32 {
    let mut x: i32 = 5;
    x += 1;
    return x;  // Should return 6
}

// Pre-increment
fn pre_increment_test() -> i32 {
    let mut x: i32 = 5;
    x += 1;
    return x;  // Should return 6
}

// Post-decrement
fn post_decrement_test() -> i32 {
    let mut x: i32 = 5;
    x -= 1;
    return x;  // Should return 4
}

// Pre-decrement
fn pre_decrement_test() -> i32 {
    let mut x: i32 = 5;
    x -= 1;
    return x;  // Should return 4
}

// Increment in for loop
fn sum_to_n(n: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i: i32;
    i = 0;
    while i < n {
        sum += i;
        i += 1;
    }
    return sum;
}

// Decrement in for loop
fn countdown_sum(n: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i: i32;
    i = n;
    while i > 0 {
        sum += i;
        i -= 1;
    }
    return sum;
}

// Multiple increments in loop body - transpiled to safe slice indexing
fn traverse_array(arr: &[i32], size: i32) {
    let mut idx: usize = 0;
    let mut i: i32;
    i = 0;
    while i < size {
        idx += 1;
        i += 1;
    }
}
