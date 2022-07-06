fn main() {
    let mut x0 = 42; // Tag: 0
                     // [Unique(0)]
    let x1 = &mut x0; // Tag: 1
                      // [Unique(1), Unique(0)]
    let xraw = x1 as *mut i32; // Tag: _
                               // [SharedRW(_), Unique(1), Unique(0)]
    let x2 = unsafe { &mut *xraw }; // Tag: 2
                                    // [Unique(2), SharedRW(_), Unique(1), Unique(0)]
    let x3 = unsafe { &mut *xraw }; // Tag: 3
                                    // N.B.: Unique(2) gets popped!
                                    // [Unique(3), SharedRW(_), Unique(1), Unique(0)]
    *x2 = 10; // Undefined behavior! Tag 2 does not have write access
    *x3 = 20;
}
