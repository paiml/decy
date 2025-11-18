/* K&R C Chapter 3.3: Else-If
 * Page 58
 * Binary search using else-if
 * Transpiled to safe Rust
 */

// Binary search: find x in v[0] <= v[1] <= ... <= v[n-1]
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
            return mid as i32;  // found match
        }
    }

    return -1;  // no match
}

fn main() {
    let arr: [i32; 11] = [2, 5, 8, 12, 16, 23, 38, 45, 56, 67, 78];
    let n: usize = arr.len();

    print!("Binary search in array: ");
    for i in 0..n {
        print!("{} ", arr[i]);
    }
    println!("\n");

    // Test searches
    let searches: [i32; 5] = [23, 45, 1, 100, 8];
    for i in 0..5 {
        let result = binsearch(searches[i], &arr);
        if result >= 0 {
            println!("Found {} at index {}", searches[i], result);
        } else {
            println!("{} not found", searches[i]);
        }
    }
}
