fn buffer_fill<'a>(buffer: &'a i32, size: i32, value: i32) {
    let mut i: i32 = 0;
    i = 0;
while i < size {
    buffer[i] = value;
    i = i + 1;
}
}
fn buffer_sum<'a>(buffer: &'a i32, size: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    sum = 0;
    i = 0;
while i < size {
    sum = sum + buffer[i];
    i = i + 1;
}
    return sum;
}
fn buffer_find<'a>(buffer: &'a i32, size: i32, target: i32) -> i32 {
    let mut i: i32 = 0;
    i = 0;
while i < size {
    if buffer[i] == target {
    return i;
}
    i = i + 1;
}
    return -1;
}
fn buffer_reverse<'a>(buffer: &'a i32, size: i32) {
    let mut i: i32 = 0;
    let mut temp: i32 = 0;
    i = 0;
while i < (size / 2) {
    temp = buffer[i];
    buffer[i] = buffer[(size - 1) - i];
    buffer[(size - 1) - i] = temp;
    i = i + 1;
}
}
