fn sum_array<'a>(mut arr: &[i32]) -> i32 {
    let mut sum: i32 = 0;
    let mut end: *mut i32 = arr + arr.len() as i32;
    while arr < end {
    sum = sum + *arr;
    arr = arr + 1;
}
    return sum;
}
fn find_first<'a>(mut arr: &[i32], mut target: i32) -> i32 {
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if arr[i as usize] == target {
    return i;
}
    i = i + 1;
}
    return -1;
}
fn count_even<'a>(mut arr: &[i32]) -> i32 {
    let mut count: i32 = 0;
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if (arr[i as usize] % 2) == 1 {
    continue;
}
    count = count + 1;
    i = i + 1;
}
    return count;
}
fn linear_search<'a>(mut arr: &[i32], mut target: i32) -> i32 {
    let mut found: i32 = 0;
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if arr[i as usize] == target {
    found = 1;
    break;
}
    i = i + 1;
}
    return found;
}
fn string_length(str: &[u8]) -> i32 {
    let mut str_idx: usize = 0;
    let mut start: *mut u8 = str;
    while str[str_idx] != 0u8 {
    str_idx += 1 as usize;
}
    return str - start;
}
