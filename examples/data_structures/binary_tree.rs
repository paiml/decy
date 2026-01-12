#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct _IO_FILE {
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeNode {
    pub value: i32,
    pub left: *mut TreeNode,
    pub right: *mut TreeNode,
}
static mut ERRNO: i32 = 0;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
pub type FILE = _IO_FILE;
// type TreeNode = TreeNode; (redundant in Rust)
fn atof(mut nptr: &str) -> f64 {
    return 0.0;
}
fn fgetc(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fprintf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn srand(mut seed: u32) {
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
fn scanf(mut format: &str) -> i32 {
    return 0;
}
fn main() {
    let mut root: *mut TreeNode = std::ptr::null_mut();
    root = insert(root, 50);
    root = insert(root, 30);
    root = insert(root, 70);
    root = insert(root, 20);
    root = insert(root, 40);
    let mut found: i32 = search(root, 40);
    print!("Found 40: {}\n", found);
    found = search(root, 100);
    print!("Found 100: {}\n", found);
    free_tree(root);
    std::process::exit(0);
}
fn ftell(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn getenv(mut name: &str) -> *mut u8 {
    return std::ptr::null_mut();
}
fn printf(mut format: &str) -> i32 {
    return 0;
}
fn rewind(stream: &mut _IO_FILE) {
}
fn abs(mut j: i32) -> i32 {
    return 0;
}
fn insert(mut root: *mut TreeNode, mut value: i32) -> *mut TreeNode {
    if root == std::ptr::null_mut() {
    return create_node(value);
}
    if value < /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).value } {
    // SAFETY: pointer is non-null and points to valid struct with exclusive access
    unsafe { (*root).left = insert(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).left }, value); }
} else {
    // SAFETY: pointer is non-null and points to valid struct with exclusive access
    unsafe { (*root).right = insert(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).right }, value); }
}
    return root;
}
fn fread(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn puts(mut s: &str) -> i32 {
    return 0;
}
fn malloc(mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn calloc(mut nmemb: size_t, mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn fgets<'a>(mut s: &[u8], stream: &mut _IO_FILE) -> *mut u8 {
    return std::ptr::null_mut();
}
fn fputs(mut s: &str, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fseek(stream: &mut _IO_FILE, mut offset: i32, mut whence: i32) -> i32 {
    return 0;
}
fn sscanf(mut str: &str, mut format: &str) -> i32 {
    return 0;
}
fn strtol(mut nptr: &str, endptr: &mut *mut u8, mut base: i32) -> i32 {
    return 0;
}
fn fclose(stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn strtod(mut nptr: &str, endptr: &mut *mut u8) -> f64 {
    return 0.0;
}
fn exit(mut status: i32) {
}
fn atoi(mut nptr: &str) -> i32 {
    return 0;
}
fn realloc(ptr: *mut (), mut size: size_t) -> *mut () {
    return std::ptr::null_mut();
}
fn free(ptr: *mut ()) {
}
fn fopen(mut filename: &str, mut mode: &str) -> *mut _IO_FILE {
    return std::ptr::null_mut();
}
fn system(mut command: &str) -> i32 {
    return 0;
}
fn fputc(mut c: i32, stream: &mut _IO_FILE) -> i32 {
    return 0;
}
fn fscanf(stream: &mut _IO_FILE, mut format: &str) -> i32 {
    return 0;
}
fn putchar(mut c: i32) -> i32 {
    return 0;
}
fn snprintf(str: &mut u8, mut size: size_t, mut format: &str) -> i32 {
    return 0;
}
fn sprintf(str: &mut u8, mut format: &str) -> i32 {
    return 0;
}
fn search(mut root: *mut TreeNode, mut value: i32) -> i32 {
    if root == std::ptr::null_mut() {
    return 0;
}
    if value == /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).value } {
    return 1;
} else {
    return search(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).left }, value);
    return search(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).right }, value);
}
}
fn abort() {
}
fn fwrite(ptr: *mut (), mut size: size_t, mut nmemb: size_t, stream: &mut _IO_FILE) -> u32 {
    return 0;
}
fn free_tree(mut root: *mut TreeNode) {
    if root == std::ptr::null_mut() {
    return;
}
    free_tree(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).left });
    free_tree(/* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*root).right });
    drop(root);
}
fn rand() -> i32 {
    return 0;
}
fn atol(mut nptr: &str) -> i32 {
    return 0;
}
fn getchar() -> i32 {
    return 0;
}
fn labs(mut j: i32) -> i32 {
    return 0;
}
