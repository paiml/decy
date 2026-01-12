#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Node {
    pub data: i32,
    pub next: *mut Node,
}
static mut ERRNO: i32 = 0;
fn create_node(mut value: i32) -> *mut Node {
    let mut node: *mut Node = std::ptr::null_mut();
    node = Box::into_raw(Box::<Node>::default());
    // SAFETY: pointer is non-null and points to valid struct with exclusive access
    unsafe { (*node).data = value; }
    // SAFETY: pointer is non-null and points to valid struct with exclusive access
    unsafe { (*node).next = std::ptr::null_mut(); }
    return node;
}
fn list_length(mut head: *mut Node) -> i32 {
    let mut count: i32 = 0i32;
    count = 0;
    while head != std::ptr::null_mut() {
    count = count + 1;
    head = /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*head).next };
}
    return count;
}
fn list_sum(mut head: *mut Node) -> i32 {
    let mut sum: i32 = 0i32;
    sum = 0;
    while head != std::ptr::null_mut() {
    sum = sum + /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*head).data };
    head = /* SAFETY: pointer is non-null and points to valid struct */ unsafe { (*head).next };
}
    return sum;
}
