#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Entry {
    pub key: *mut u8,
    pub value: i32,
    pub next: *mut Entry,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashTable {
    pub buckets: [*mut Entry; 100],
}
static mut ERRNO: i32 = 0;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
pub type FILE = _IO_FILE;
// type Entry = Entry; (redundant in Rust)
// type HashTable = HashTable; (redundant in Rust)
fn main() {
    let mut table: *mut HashTable = create_table();
    insert(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table }, "apple", 5);
    insert(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table }, "banana", 7);
    insert(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table }, "cherry", 3);
    let mut value: i32 = 0i32;
    if (get(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table }, "banana", &mut value)) != 0 {
    print!("banana: {}\n", value);
}
    if (get(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table }, "grape", &mut value) == 0) {
    print!("grape not found\n");
}
    free_table(/* SAFETY: pointer is non-null and valid for the duration of the call */ unsafe { &mut *table });
    std::process::exit(0);
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn atof(mut nptr: &str) -> f64 {
    return 0.0;
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn strlen(mut s: &str) -> u32 {
    return 0;
}
fn fread(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn strncmp(mut s1: &str, mut s2: &str, mut n: size_t) -> i32 {
    return 0;
}
fn strtod(mut nptr: &str, endptr: &mut *mut u8) -> f64 {
    return 0.0;
}
fn insert(table: &mut HashTable, mut key: &str, mut value: i32) {
    let mut index: u32 = hash(key);
    let mut entry: Box<Entry> = Box::default();
    entry.key = Box::leak(vec![0u8; (key.len() as i32 + 1) as usize].into_boxed_slice()).as_mut_ptr();
    key.to_string();
    entry.value = value;
    entry.next = (*table).buckets[(index) as usize];
    (*table).buckets[(index) as usize] = Box::into_raw(entry);
}
fn atol(mut nptr: &str) -> i32 {
    return 0;
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn memmove(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn strncpy(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn atoi(mut nptr: &str) -> i32 {
    return 0;
}
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn strtok(str: &mut u8, mut delim: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn memset(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn hash(mut key: &str) -> u32 {
    let mut hash: u32 = 0;
    while !key.is_empty() {
    hash = (hash << 5) + { let __tmp = key.as_bytes()[0] as u32; key = &key[1..]; __tmp };
}
    return hash % 100;
}
fn memchr(s: *mut (), mut c: i32, mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn getenv(mut name: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strstr(mut haystack: &str, mut needle: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strncat(dest: &mut u8, mut src: &str, mut n: size_t) -> *mut u8 {
    return std::ptr::null_mut();
}
fn labs(mut j: i32) -> i32 {
    return 0;
}
fn strchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn strcat(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn rand() -> i32 {
    return 0;
}
fn strrchr(mut s: &str, mut c: i32) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fwrite(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn strtol(mut nptr: &str, endptr: &mut *mut u8, mut base: i32) -> i32 {
    return 0;
}
fn snprintf(str: &mut u8, mut size: size_t, mut format: &str) -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn get<'a>(table: &mut HashTable, mut key: &str, mut value: &'a mut i32) -> i32 {
    let mut index: u32 = hash(key);
    let mut entry: *mut Entry = (*table).buckets[(index) as usize];
    while entry != std::ptr::null_mut() {
    if strcmp(/* SAFETY: string pointer is null-terminated and valid */ unsafe { std::ffi::CStr::from_ptr((*entry).key as *const i8).to_str().unwrap_or("") }, key) == 0 {
    *value = /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*entry).value };
    return 1;
}
    entry = /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*entry).next };
}
    return 0;
}
fn strcpy(dest: &mut u8, mut src: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn calloc(mut nmemb: size_t, mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn strcmp(mut s1: &str, mut s2: &str) -> i32 {
    return 0;
}
fn strdup(mut s: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn srand(mut seed: u32) {
}
fn free_table(table: &mut HashTable) {
    let mut i: i32 = 0;
while i < 100 {
    let mut entry: *mut Entry = (*table).buckets[(i) as usize];
    while entry != std::ptr::null_mut() {
    let mut next: *mut Entry = /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*entry).next };
    drop(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*entry).key });
    drop(entry);
    entry = next;
}
    i = i + 1;
}
    drop(table);
}
fn abs(mut j: i32) -> i32 {
    return 0;
}
fn realloc(ptr: *mut (), mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn memcmp(s1: *mut (), s2: *mut (), mut n: size_t) -> i32 {
    return 0;
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn malloc(mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn exit(mut status: i32) {
}
fn system(mut command: &str) -> i32 {
    return 0;
}
fn abort() {
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn getchar() -> i32 {
    return 0;
}
fn create_table() -> *mut HashTable {
    let mut table: Box<HashTable> = Box::new(/* SAFETY: HashTable is valid when zero-initialized */ unsafe { std::mem::zeroed::<HashTable>() });
    if true /* Box never null */ {
    let mut i: i32 = 0;
while i < 100 {
    (*table).buckets[(i) as usize] = std::ptr::null_mut();
    i = i + 1;
}
}
    return Box::into_raw(table);
}
fn free(ptr: *mut ()) {
}
fn rewind(stream: &mut _IO_FILE) {
}
fn memcpy(dest: *mut (), src: *mut (), mut n: size_t) -> *mut () {
    return std::ptr::null_mut();
}
