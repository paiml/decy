fn absolute(mut x: i32) -> i32 {
    if x < 0 {
    return -x;
} else {
    return x;
}
}
fn power(mut base: i32, mut exp: i32) -> i32 {
    let mut result: i32 = 0i32;
    let mut i: i32 = 0i32;
    result = 1;
    i = 0;
while i < exp {
    result = result * base;
    i = i + 1;
}
    return result;
}
fn gcd(mut a: i32, mut b: i32) -> i32 {
    let mut temp: i32 = 0i32;
    while b != 0 {
    temp = b;
    b = a % b;
    a = temp;
}
    return a;
}
fn is_prime(mut n: i32) -> i32 {
    let mut i: i32 = 0i32;
    if n <= 1 {
    return 0;
}
    i = 2;
while (i * i) <= n {
    if (n % i) == 0 {
    return 0;
}
    i = i + 1;
}
    return 1;
}
