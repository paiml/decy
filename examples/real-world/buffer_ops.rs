static mut ERRNO: i32 = 0;
fn buffer_reverse<'a>(mut buffer: &mut [i32]) {
    let mut i: i32 = 0i32;
    let mut temp: i32 = 0i32;
    i = 0;
while i < (buffer.len() as i32 / 2) {
    temp = buffer[(i) as usize];
    buffer[(i) as usize] = buffer[((buffer.len() as i32 - 1) - i) as usize];
    buffer[((buffer.len() as i32 - 1) - i) as usize] = temp;
    i = i + 1;
}
}
fn buffer_find<'a>(mut buffer: &[i32], mut target: i32) -> i32 {
    let mut i: i32 = 0i32;
    i = 0;
while i < buffer.len() as i32 {
    if buffer[(i) as usize] == target {
    return i;
}
    i = i + 1;
}
    return -1;
}
fn buffer_sum<'a>(mut buffer: &[i32]) -> i32 {
    let mut sum: i32 = 0i32;
    let mut i: i32 = 0i32;
    sum = 0;
    i = 0;
while i < buffer.len() as i32 {
    sum = sum + buffer[(i) as usize];
    i = i + 1;
}
    return sum;
}
fn buffer_fill<'a>(mut buffer: &mut [i32], mut value: i32) {
    let mut i: i32 = 0i32;
    i = 0;
while i < buffer.len() as i32 {
    buffer[(i) as usize] = value;
    i = i + 1;
}
}
