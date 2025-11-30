#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    pub key: *mut u8,
    pub value: i32,
    pub next: *mut Entry,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashTable {
    pub buckets: [*mut Entry; 100],
}
pub type size_t = i32;
pub type ssize_t = i32;
pub type ptrdiff_t = i32;
pub type FILE = _IO_FILE;
// type Entry = Entry; (redundant in Rust)
// type HashTable = HashTable; (redundant in Rust)
fn abort() {
}
fn abs(mut j: i32) -> i32 {
    return 0;
}
fn atof(mut nptr: &str) -> f64 {
    return 0.0;
}
fn atoi(mut nptr: &str) -> i32 {
    return 0;
}
fn atol(mut nptr: &str) -> i32 {
    return 0;
}
fn calloc(mut nmemb: i32, mut size: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn exit(mut status: i32) {
}
fn free(ptr: &mut ()) {
}
fn getenv(mut name: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn labs(mut j: i32) -> i32 {
    return 0;
}
fn malloc(mut size: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn rand() -> i32 {
    return 0;
}
fn realloc<'a>(mut ptr: &[()]) -> *mut () {
    return std::ptr::null_mut();
}
fn srand(mut seed: i32) {
}
fn strtod(mut nptr: &str, endptr: &mut *mut u8) -> f64 {
    return 0.0;
}
fn strtol(mut nptr: &str, endptr: &mut *mut u8, mut base: i32) -> i32 {
    return 0;
}
fn system(mut command: &str) -> i32 {
    return 0;
}
fn memchr(s: &mut (), mut c: i32, mut n: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn memcmp(s1: &mut (), s2: &mut (), mut n: i32) -> i32 {
    return 0;
}
fn memcpy(dest: &mut (), src: &mut (), mut n: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn memmove(dest: &mut (), src: &mut (), mut n: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn memset(s: &mut (), mut c: i32, mut n: i32) -> *mut () {
    return std::ptr::null_mut();
}
fn strcat(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strcmp(mut s1: &str, mut s2: &str) -> i32 {
    return 0;
}
fn strcpy(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strlen(mut s: &str) -> i32 {
    return 0;
}
fn strncat(dest: &mut u8, mut src: &str, mut n: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strncmp(mut s1: &str, mut s2: &str, mut n: i32) -> i32 {
    return 0;
}
fn strncpy(dest: &mut u8, mut src: &str, mut n: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strrchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strstr(mut haystack: &str, mut needle: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strtok(str: &mut u8, mut delim: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fread<'a>(mut ptr: &[()], mut nmemb: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fwrite<'a>(mut ptr: &[()], mut nmemb: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn getchar() -> i32 {
    return 0;
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn rewind(stream: &mut _IO_FILE) {
}
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn snprintf<'a>(mut str: &[u8], mut format: &str) -> i32 {
    return 0;
}
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn hash(mut key: &str) -> i32 {
    let mut hash: i32 = 0;
    while !key.is_empty() {
    hash = (hash << 5) + { let __tmp = key.as_bytes()[0] as i32; key = &key[1..]; __tmp };
}
    return hash % 100;
}
fn create_table() -> *mut HashTable {
    let mut table: Box<HashTable> = Box::new(unsafe { std::mem::zeroed::<HashTable>() });
    if true /* Box never null */ {
    let mut i: i32 = 0;
while i < 100 {
    (*table).buckets[i as usize] = std::ptr::null_mut();
    i = i + 1;
}
}
    return Box::into_raw(table);
}
fn insert(table: &mut HashTable, mut key: &str, mut value: i32) {
    let mut index: i32 = hash(key);
    let mut entry: Box<Entry> = Box::default();
    entry.key = Box::leak(vec![0u8; key.len() + 1 as usize].into_boxed_slice()).as_mut_ptr();
    key.to_string();
    entry.value = value;
    entry.next = (*table).buckets[index as usize];
    (*table).buckets[index as usize] = Box::into_raw(entry);
}
fn get(table: &mut HashTable, mut key: &str, value: &mut i32) -> i32 {
    let mut index: i32 = hash(key);
    let mut entry: *mut Entry = (*table).buckets[index as usize];
    while entry != std::ptr::null_mut() {
    if strcmp(unsafe { std::ffi::CStr::from_ptr((*entry).key as *const i8).to_str().unwrap_or("") }, key) == 0 {
    *value = unsafe { (*entry).value };
    return 1;
}
    entry = unsafe { (*entry).next };
}
    return 0;
}
fn free_table(table: &mut HashTable) {
    let mut i: i32 = 0;
while i < 100 {
    let mut entry: *mut Entry = (*table).buckets[i as usize];
    while entry != std::ptr::null_mut() {
    let mut next: *mut Entry = unsafe { (*entry).next };
    drop(unsafe { (*entry).key });
    drop(entry);
    entry = next;
}
    i = i + 1;
}
    drop(table);
}
fn main() {
    let mut table: *mut HashTable = create_table();
    insert(unsafe { &mut *table }, "apple", 5);
    insert(unsafe { &mut *table }, "banana", 7);
    insert(unsafe { &mut *table }, "cherry", 3);
    let mut value: i32 = 0i32;
    if (get(unsafe { &mut *table }, "banana", &mut value)) != 0 {
    print!("banana: {}\n", value);
}
    if get(unsafe { &mut *table }, "grape", &mut value) == 0 {
    print!("grape not found\n");
}
    free_table(unsafe { &mut *table });
    std::process::exit(0);
}
