#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
static mut ERRNO: i32 = 0;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
pub type FILE = _IO_FILE;
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn isdigit(mut c: i32) -> i32 {
    return 0;
}
fn isupper(mut c: i32) -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn main(mut argc: i32, argv: &mut *mut u8) {
    if argc != 2 {
    { use std::io::Write; write!(std::io::stderr(), "Usage: {} <filename>\n", unsafe { std::ffi::CStr::from_ptr(argv[(0) as usize] as *const i8).to_str().unwrap_or("") }).map(|_| 0).unwrap_or(-1) };
    std::process::exit(1);
}
    let mut count: i32 = count_words(argv[(1) as usize]);
    if count < 0 {
    { use std::io::Write; write!(std::io::stderr(), "Error: Could not open file {}\n", unsafe { std::ffi::CStr::from_ptr(argv[(1) as usize] as *const i8).to_str().unwrap_or("") }).map(|_| 0).unwrap_or(-1) };
    std::process::exit(1);
}
    print!("{}\n", count);
    std::process::exit(0);
}
fn isalnum(mut c: i32) -> i32 {
    return 0;
}
fn rewind(stream: &mut _IO_FILE) {
}
fn count_words(mut filename: &str) -> i32 {
    let mut fp: *mut _IO_FILE = std::fs::File::open(filename).ok();
    if fp == std::ptr::null_mut() {
    return -1;
}
    let mut word_count: i32 = 0;
    let mut in_word: i32 = 0;
    let mut c: i32 = 0i32;
    while ({ let __assign_tmp = { use std::io::Read; let mut buf = [0u8; 1]; fp.read(&mut buf).map(|_| buf[0] as i32).unwrap_or(-1) }; c = __assign_tmp; __assign_tmp }) != -1 {
    if (isspace(c)) != 0 {
    in_word = 0;
} else {
    in_word = 1;
    word_count = word_count + 1;
}
}
    drop(fp);
    return word_count;
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn snprintf(str: &mut u8, mut size: size_t, mut format: &str) -> i32 {
    return 0;
}
fn isalpha(mut c: i32) -> i32 {
    return 0;
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn islower(mut c: i32) -> i32 {
    return 0;
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn tolower(mut c: i32) -> i32 {
    return 0;
}
fn fread(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn toupper(mut c: i32) -> i32 {
    return 0;
}
fn getchar() -> i32 {
    return 0;
}
fn fwrite(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn isspace(mut c: i32) -> i32 {
    return 0;
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
