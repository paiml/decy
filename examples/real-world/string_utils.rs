fn string_length<'a>(str: &'a u8) -> i32 {
    let mut len: i32 = 0;
    len = 0;
    while *str != 0 {
    len = len + 1;
    str = str + 1;
}
    return len;
}
fn string_compare<'a>(s1: &'a u8, s2: &'a u8) -> i32 {
    while (*s1 != 0) && (*s2 != 0) {
    if *s1 != *s2 {
    return *s1 - *s2;
}
    s1 = s1 + 1;
    s2 = s2 + 1;
}
    return *s1 - *s2;
}
fn string_copy<'a>(dest: &'a u8, src: &'a u8) {
    while *src != 0 {
    *dest = *src;
    dest = dest + 1;
    src = src + 1;
}
    *dest = 0;
}
