static mut ERRNO: i32 = 0;
fn string_length(str: &[u8]) -> i32 {
    let mut str_idx: usize = 0;
    let mut len: i32 = 0i32;
    len = 0;
    while str[str_idx] != 0 {
    len = len + 1;
    str_idx += 1 as usize;
}
    return len;
}
fn string_copy(dest: &mut [u8], src: &[u8]) {
    let mut dest_idx: usize = 0;
    let mut src_idx: usize = 0;
    while src[src_idx] != 0 {
    dest[dest_idx] = src[src_idx];
    dest_idx += 1 as usize;
    src_idx += 1 as usize;
}
    dest[dest_idx] = 0;
}
fn string_compare(s1: &[u8], s2: &[u8]) -> i32 {
    let mut s1_idx: usize = 0;
    let mut s2_idx: usize = 0;
    while (s1[s1_idx] != 0) && (s2[s2_idx] != 0) {
    if s1[s1_idx] != s2[s2_idx] {
    return (s1[s1_idx] as i32) - (s2[s2_idx] as i32);
}
    s1_idx += 1 as usize;
    s2_idx += 1 as usize;
}
    return (s1[s1_idx] as i32) - (s2[s2_idx] as i32);
}
