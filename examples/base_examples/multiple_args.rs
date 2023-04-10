fn basic_example(x: &mut i32, w: i32, z: i32, y: &mut i32) -> i32 {
    *x = 42 + z;
    *y = 13 + w;
    return *x; // Has to read 42 , because x and y cannot alias !
}

fn main() {
    let mut local = 5;
    let w = 1;
    let z = 2;
    let raw_pointer = &mut local as *mut i32;
    let result = unsafe { basic_example(&mut *raw_pointer, w, z, &mut *raw_pointer) };
}
