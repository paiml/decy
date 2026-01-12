#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
static mut ERRNO: i32 = 0;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
pub type FILE = _IO_FILE;
fn strrchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn memchr(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn strchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strncpy(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strtok(str: &mut u8, mut delim: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn grep_file(mut pattern: &str, mut filename: &str, mut match_count: *mut i32) -> i32 {
    let mut fp: *mut _IO_FILE = std::fs::File::open(filename).ok();
    if fp == std::ptr::null_mut() {
    return -1;
}
    let mut line: [u8; 1024] = [0u8; 1024];
    // SAFETY: pointer is valid, aligned, and not aliased during write
    unsafe { *match_count = 0; }
    while fgets(&line, fp) != 0 {
    if strstr(line, pattern) != 0 {
    print!("{}", unsafe { std::ffi::CStr::from_ptr(line.as_ptr() as *const i8).to_str().unwrap_or("") });
    match_count = match_count.wrapping_add(1 as usize);
}
}
    drop(fp);
    return 0;
}
fn getchar() -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn strncat(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn rewind(stream: &mut _IO_FILE) {
}
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn fread(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn strcmp(mut s1: &str, mut s2: &str) -> i32 {
    return 0;
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn memcmp(s1: *mut (), s2: *mut (), mut n: size_t) -> i32 {
    return 0;
}
fn strcpy(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strdup(mut s: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strlen(mut s: &str) -> u32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn memcpy(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn strstr(mut haystack: &str, mut needle: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn main(mut argc: i32, argv: &mut *mut u8) {
    if argc != 3 {
    { use std::io::Write; write!(std::io::stderr(), "Usage: {} <pattern> <filename>\n", unsafe { std::ffi::CStr::from_ptr(argv[(0) as usize] as *const i8).to_str().unwrap_or("") }).map(|_| 0).unwrap_or(-1) };
    std::process::exit(1);
}
    let mut matches: i32 = 0;
    if grep_file(argv[(1) as usize], argv[(2) as usize], &matches) != 0 {
    { use std::io::Write; write!(std::io::stderr(), "Error: Could not open file {}\n", unsafe { std::ffi::CStr::from_ptr(argv[(2) as usize] as *const i8).to_str().unwrap_or("") }).map(|_| 0).unwrap_or(-1) };
    std::process::exit(1);
}
    { use std::io::Write; write!(std::io::stderr(), "Found {} matching lines\n", matches).map(|_| 0).unwrap_or(-1) };
    std::process::exit(0);
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn memmove(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn memset(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn fwrite(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn snprintf(str: &mut u8, mut size: size_t, mut format: &str) -> i32 {
    return 0;
}
fn strcat(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn strncmp(mut s1: &str, mut s2: &str, mut n: size_t) -> i32 {
    return 0;
}
