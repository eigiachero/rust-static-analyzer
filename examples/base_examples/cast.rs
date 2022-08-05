use std::mem::size_of_val;

pub fn main() {
    let mut one: u64 = 5;
    let raw = &mut one as *mut u64;
    let raw2 = raw as *mut u32;
    unsafe {
        let two = *raw2;
        //println!("{} {} {}", *two, size_of_val(&*raw), size_of_val(&*two));
    }
}