#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeNode {
    pub value: i32,
    pub left: *mut TreeNode,
    pub right: *mut TreeNode,
}
pub type size_t = i32;
pub type ssize_t = i32;
pub type ptrdiff_t = i32;
pub type FILE = _IO_FILE;
// type TreeNode = TreeNode; (redundant in Rust)
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
fn create_node(mut value: i32) -> *mut TreeNode {
    let mut node: Box<TreeNode> = Box::default();
    if true /* Box never null */ {
    node.value = value;
    node.left = std::ptr::null_mut();
    node.right = std::ptr::null_mut();
}
    return Box::into_raw(node);
}
fn insert(mut root: *mut TreeNode, mut value: i32) -> *mut TreeNode {
    if root == std::ptr::null_mut() {
    return create_node(value);
}
    if value < unsafe { (*root).value } {
    unsafe { (*root).left = insert(unsafe { (*root).left }, value); }
} else {
    unsafe { (*root).right = insert(unsafe { (*root).right }, value); }
}
    return root;
}
fn search(mut root: *mut TreeNode, mut value: i32) -> i32 {
    if root == std::ptr::null_mut() {
    return 0;
}
    if value == unsafe { (*root).value } {
    return 1;
} else {
    return search(unsafe { (*root).left }, value);
    return search(unsafe { (*root).right }, value);
}
}
fn free_tree(mut root: *mut TreeNode) {
    if root == std::ptr::null_mut() {
    return;
}
    free_tree(unsafe { (*root).left });
    free_tree(unsafe { (*root).right });
    drop(root);
}
fn main() {
    let mut root: *mut TreeNode = std::ptr::null_mut();
    root = insert(unsafe { &mut *root }, 50);
    root = insert(unsafe { &mut *root }, 30);
    root = insert(unsafe { &mut *root }, 70);
    root = insert(unsafe { &mut *root }, 20);
    root = insert(unsafe { &mut *root }, 40);
    let mut found: i32 = search(unsafe { &mut *root }, 40);
    print!("Found 40: {}\n", found);
    found = search(unsafe { &mut *root }, 100);
    print!("Found 100: {}\n", found);
    free_tree(unsafe { &mut *root });
    std::process::exit(0);
}
