/* K&R C Chapter 3.3: Else-If
 * Page 53-54
 * Binary search example with else-if
 * Transpiled to safe Rust
 */

// binsearch: find x in v[0] <= v[1] <= ... <= v[n-1]
fn binsearch(x: i32, v: &[i32]) -> i32 {
    let mut low: usize;
    let mut high: usize;
    let mut mid: usize;

    low = 0;
    high = v.len() - 1;
    while low <= high {
        mid = (low + high) / 2;
        if x < v[mid] {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        } else if x > v[mid] {
            low = mid + 1;
        } else {
            return mid as i32;
        }
    }
    return -1;
}

fn main() {
    let arr: [i32; 10] = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    let x: i32 = 7;
    let result: i32;

    result = binsearch(x, &arr);
    println!("binsearch({}) = {}", x, result);
}
