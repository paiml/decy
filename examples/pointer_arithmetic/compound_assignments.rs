// DECY-041: Test compound assignment operators
// Transpiled to idiomatic Rust

fn increment_by(mut value: i32, amount: i32) -> i32 {
    value += amount;
    return value;
}

fn decrement_by(mut value: i32, amount: i32) -> i32 {
    value -= amount;
    return value;
}

fn multiply_by(mut value: i32, factor: i32) -> i32 {
    value *= factor;
    return value;
}

fn divide_by(mut value: i32, divisor: i32) -> i32 {
    if divisor != 0 {
        value /= divisor;
    }
    return value;
}

fn modulo_by(mut value: i32, modulus: i32) -> i32 {
    if modulus != 0 {
        value %= modulus;
    }
    return value;
}

// Pointer arithmetic with compound assignments - transpiled to safe slice indexing
fn advance_pointer(arr: &[i32], offset: usize) -> &[i32] {
    if offset < arr.len() {
        return &arr[offset..];
    }
    return &arr[0..0];
}
