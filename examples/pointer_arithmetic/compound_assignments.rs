static mut ERRNO: i32 = 0;
fn decrement_by(mut value: i32, mut amount: i32) -> i32 {
    value = value - amount;
    return value;
}
fn increment_by(mut value: i32, mut amount: i32) -> i32 {
    value = value + amount;
    return value;
}
fn divide_by(mut value: i32, mut divisor: i32) -> i32 {
    if divisor != 0 {
    value = value / divisor;
}
    return value;
}
fn modulo_by(mut value: i32, mut modulus: i32) -> i32 {
    if modulus != 0 {
    value = value % modulus;
}
    return value;
}
fn advance_pointer(mut ptr: *mut i32, mut offset: i32) {
    ptr = ptr.wrapping_add(offset as usize);
}
fn multiply_by(mut value: i32, mut factor: i32) -> i32 {
    value = value * factor;
    return value;
}
