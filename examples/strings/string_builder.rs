#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StringBuilder {
    pub data: *mut u8,
    pub length: size_t,
    pub capacity: size_t,
}
static mut ERRNO: i32 = 0;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
pub type FILE = _IO_FILE;
fn getchar() -> i32 {
    return 0;
}
fn snprintf(str: &mut u8, mut size: size_t, mut format: &str) -> i32 {
    return 0;
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn rand() -> i32 {
    return 0;
}
fn sb_append_char(sb: &mut StringBuilder, mut c: u8) {
    if ((*sb).length + 2) > (*sb).capacity {
    (*sb).capacity = (*sb).capacity * 2;
    sb.data = realloc((*sb).data as *mut (), (*sb).capacity) as *mut u8;
}
    // SAFETY: index is within bounds of allocated array
    unsafe { *(*sb).data.add(((*sb).length) as usize) = c; }
    sb.length = (*sb).length + 1;
    // SAFETY: index is within bounds of allocated array
    unsafe { *(*sb).data.add(((*sb).length) as usize) = 0u8; }
}
fn strcmp(mut s1: &str, mut s2: &str) -> i32 {
    return 0;
}
fn abort() {
}
fn strdup(mut s: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn exit(mut status: i32) {
}
fn atof(mut nptr: &str) -> f64 {
    return 0.0;
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn strncpy(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn sb_append(sb: &mut StringBuilder, mut str: &str) {
    let mut str_len: size_t = str.len() as i32;
    let mut new_len: size_t = (*sb).length + str_len;
    if (new_len + 1) > (*sb).capacity {
    while (*sb).capacity <= new_len {
    (*sb).capacity = (*sb).capacity * 2;
}
    sb.data = realloc((*sb).data as *mut (), (*sb).capacity) as *mut u8;
}
    str.to_string();
    sb.length = new_len;
}
fn main() {
    let mut sb: *mut StringBuilder = sb_create();
    sb_append(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb }, "Hello");
    sb_append_char(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb }, b' ');
    sb_append(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb }, "World");
    sb_append_char(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb }, b'!');
    print!("{}\n", unsafe { std::ffi::CStr::from_ptr(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*sb).data } as *const i8).to_str().unwrap_or("") });
    print!("Length: {}, Capacity: {}\n", /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*sb).length }, /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*sb).capacity });
    let mut result: *mut u8 = sb_to_string(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb });
    print!("Result: {}\n", unsafe { std::ffi::CStr::from_ptr(result as *const i8).to_str().unwrap_or("") });
    drop(result);
    sb_free(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *sb });
    std::process::exit(0);
}
fn memmove(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn strtok(str: &mut u8, mut delim: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn strtod(mut nptr: &str, endptr: &mut *mut u8) -> f64 {
    return 0.0;
}
fn strrchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn calloc(mut nmemb: size_t, mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn realloc(ptr: *mut (), mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn memchr(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn strncmp(mut s1: &str, mut s2: &str, mut n: size_t) -> i32 {
    return 0;
}
fn memcmp(s1: *mut (), s2: *mut (), mut n: size_t) -> i32 {
    return 0;
}
fn strncat(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn system(mut command: &str) -> i32 {
    return 0;
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn sb_create() -> *mut StringBuilder {
    let mut sb: Box<StringBuilder> = Box::default();
    if true /* Box never null */ {
    sb.capacity = 16;
    sb.length = 0;
    sb.data = Box::leak(vec![0u8; ((*sb).capacity) as usize].into_boxed_slice()).as_mut_ptr();
    // SAFETY: index is within bounds of allocated array
    unsafe { *(*sb).data.add((0) as usize) = 0u8; }
}
    return Box::into_raw(sb);
}
fn abs(mut j: i32) -> i32 {
    return 0;
}
fn strcpy(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn memset(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn strlen(mut s: &str) -> u32 {
    return 0;
}
fn memcpy(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn sb_free(sb: &mut StringBuilder) {
    drop((*sb).data);
    drop(sb);
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fread(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn rewind(stream: &mut _IO_FILE) {
}
fn strstr(mut haystack: &str, mut needle: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strcat(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fwrite(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn labs(mut j: i32) -> i32 {
    return 0;
}
fn malloc(mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn atol(mut nptr: &str) -> i32 {
    return 0;
}
fn free(ptr: *mut ()) {
}
fn getenv(mut name: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn srand(mut seed: u32) {
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn atoi(mut nptr: &str) -> i32 {
    return 0;
}
fn strtol(mut nptr: &str, endptr: &mut *mut u8, mut base: i32) -> i32 {
    return 0;
}
fn sb_to_string(sb: &mut StringBuilder) -> *mut u8 {
    let mut result: Vec<u8> = Vec::<u8>::with_capacity(((*sb).length + 1) as usize) as *mut u8;
    unsafe { std::ffi::CStr::from_ptr((*sb).data as *const i8).to_str().unwrap_or("").to_string() };
    return result.as_mut_ptr();
}
