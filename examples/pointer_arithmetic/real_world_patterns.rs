// DECY-041: Real-world pointer arithmetic patterns
// Transpiled to safe Rust using slices

// Array traversal with pointer arithmetic - transpiled to iterator
fn sum_array(arr: &[i32]) -> i32 {
    let mut sum: i32 = 0;
    for &val in arr {
        sum += val;
    }
    return sum;
}

// Find first occurrence in array
fn find_first(arr: &[i32], target: i32) -> i32 {
    let mut i: i32;
    i = 0;
    while (i as usize) < arr.len() {
        if arr[i as usize] == target {
            return i;
        }
        i += 1;
    }
    return -1;
}

// Count occurrences with continue
fn count_even(arr: &[i32]) -> i32 {
    let mut count: i32 = 0;
    let mut i: usize;
    i = 0;
    while i < arr.len() {
        if arr[i] % 2 == 1 {
            i += 1;
            continue;  // Skip odd numbers
        }
        count += 1;
        i += 1;
    }
    return count;
}

// Early exit with break
fn linear_search(arr: &[i32], target: i32) -> i32 {
    let mut found: i32 = 0;
    let mut i: usize;
    i = 0;
    while i < arr.len() {
        if arr[i] == target {
            found = 1;
            break;  // Early exit when found
        }
        i += 1;
    }
    return found;
}

// String length calculation (classic pointer arithmetic) - transpiled to safe slice
fn string_length(s: &str) -> i32 {
    return s.len() as i32;
}
