fn create_node(value: i32) -> *mut Node {
    let mut node: *mut Node = std::ptr::null_mut();
    node = malloc;
    node.data = value;
    node.next = 0;
    return node;
}
fn list_length<'a>(head: &'a Node) -> i32 {
    let mut count: i32 = 0;
    count = 0;
    while head != 0 {
    count = count + 1;
    head = (*head).next;
}
    return count;
}
fn list_sum<'a>(head: &'a Node) -> i32 {
    let mut sum: i32 = 0;
    sum = 0;
    while head != 0 {
    sum = sum + (*head).data;
    head = (*head).next;
}
    return sum;
}
