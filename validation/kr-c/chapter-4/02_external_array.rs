/* K&R C Chapter 4: External Arrays
 * Global array declaration example
 * Transpiled to safe Rust using static mut
 */

static mut BUF: [i32; 10] = [0; 10];

fn main() {
    let mut i: i32;
    i = 0;
    while i < 10 {
        unsafe {
            BUF[i as usize] = i * 2;
        }
        i += 1;
    }

    unsafe {
        println!("buf[5] = {}", BUF[5]);
    }
}
