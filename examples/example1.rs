fn example1(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 13;
    return *x; // Has to read 42 , because x and y cannot alias !
}

fn main() {
    let mut local = 5;
    let raw_pointer = &mut local as *mut i32;
    let mut result = unsafe { example1(&mut *raw_pointer, &mut *raw_pointer) };
    // println!(" {} ", result); // Prints "13".
    result += 1;
}
