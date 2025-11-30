fn max(mut a: i32, mut b: i32) -> i32 {
    if a > b {
    return a;
} else {
    return b;
}
}
fn factorial(mut n: i32) -> i32 {
    let mut result: i32 = 0i32;
    let mut i: i32 = 0i32;
    result = 1;
    i = 1;
while i <= n {
    result = result * i;
    i = i + 1;
}
    return result;
}
