#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Node {
    pub data: i32,
    pub next: *mut Node,
}
fn create_node(mut value: i32) -> *mut Node {
    let mut node: *mut Node = std::ptr::null_mut();
    node = std::mem::size_of::<Node>() as i32;
    node.data = value;
    node.next = std::ptr::null_mut();
    return node;
}
fn list_length(mut head: *mut Node) -> i32 {
    let mut count: i32 = 0i32;
    count = 0;
    while head != std::ptr::null_mut() {
    count = count + 1;
    head = unsafe { (*head).next };
}
    return count;
}
fn list_sum(mut head: *mut Node) -> i32 {
    let mut sum: i32 = 0i32;
    sum = 0;
    while head != std::ptr::null_mut() {
    sum = sum + unsafe { (*head).data };
    head = unsafe { (*head).next };
}
    return sum;
}
