static mut ERRNO: i32 = 0;
fn traverse_array<'a>(mut arr: &[i32]) {
    let mut ptr: *mut i32 = arr.as_ptr() as *mut i32;
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    ptr = ptr.wrapping_add(1 as usize);
    i = i + 1;
}
}
fn sum_to_n(mut n: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = 0i32;
    i = 0;
while i < n {
    sum = sum + i;
    i = i + 1;
}
    return sum;
}
fn pre_decrement_test() -> i32 {
    let mut x: i32 = 5;
    x = x - 1;
    return x;
}
fn post_increment_test() -> i32 {
    let mut x: i32 = 5;
    x = x + 1;
    return x;
}
fn pre_increment_test() -> i32 {
    let mut x: i32 = 5;
    x = x + 1;
    return x;
}
fn post_decrement_test() -> i32 {
    let mut x: i32 = 5;
    x = x - 1;
    return x;
}
fn countdown_sum(mut n: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = 0i32;
    i = n;
while i > 0 {
    sum = sum + i;
    i = i - 1;
}
    return sum;
}
