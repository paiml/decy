static mut ERRNO: i32 = 0;
fn string_length(mut str: *mut u8) -> i32 {
    let mut start: *mut u8 = str;
    while /* SAFETY: pointer is valid and properly aligned from caller contract */ unsafe { *str } != 0u8 {
    str = str.wrapping_add(1 as usize);
}
    return /* SAFETY: both pointers derive from same allocation */ unsafe { str.offset_from(start) as i32 };
}
fn count_even<'a>(mut arr: &[i32]) -> i32 {
    let mut count: i32 = 0;
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if (arr[(i) as usize] % 2) == 1 {
    continue;
}
    count = count + 1;
    i = i + 1;
}
    return count;
}
fn sum_array(mut arr: *mut i32, mut size: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut end: *mut i32 = arr.wrapping_add(size as usize);
    while arr < end {
    sum = sum + /* SAFETY: pointer is valid and properly aligned from caller contract */ unsafe { *arr };
    arr = arr.wrapping_add(1 as usize);
}
    return sum;
}
fn linear_search<'a>(mut arr: &[i32], mut target: i32) -> i32 {
    let mut found: i32 = 0;
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if arr[(i) as usize] == target {
    found = 1;
    break;
}
    i = i + 1;
}
    return found;
}
fn find_first<'a>(mut arr: &[i32], mut target: i32) -> i32 {
    let mut i: i32 = 0i32;
    i = 0;
while i < arr.len() as i32 {
    if arr[(i) as usize] == target {
    return i;
}
    i = i + 1;
}
    return -1;
}
