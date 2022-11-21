pub fn main() {
    let mut local: u32 = 1;
    let raw_1 = &mut local as *mut u32;
    let raw_2 = &mut local as *mut u32;
    let res = unsafe {auxiliar(&mut *raw_1, &mut *raw_2)};

}

pub fn auxiliar(x: &mut u32, y: &mut u32) -> u32{
    *y = 2;
    *x = 3;
    *x
}
