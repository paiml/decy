fn max(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    } else {
        return b;
    }
}

fn factorial(n: i32) -> i32 {
    let mut result: i32;
    let mut i: i32;
    result = 1;
    i = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    return result;
}
