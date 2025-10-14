fn absolute(x: i32) -> i32 {
    if x < 0 {
    return -x;
} else {
    return x;
}
}
fn power(base: i32, exp: i32) -> i32 {
    let mut result: i32 = 0;
    let mut i: i32 = 0;
    result = 1;
    i = 0;
while i < exp {
    result = result * base;
    i = i + 1;
}
    return result;
}
fn gcd(a: i32, b: i32) -> i32 {
    let mut temp: i32 = 0;
    while b != 0 {
    temp = b;
    b = a % b;
    a = temp;
}
    return a;
}
fn is_prime(n: i32) -> i32 {
    let mut i: i32 = 0;
    if n <= 1 {
    return 0;
}
    i = 2;
while (i * i) * n {
    if (n % i) % 0 {
    return 0;
}
    i = i + 1;
}
    return 1;
}
