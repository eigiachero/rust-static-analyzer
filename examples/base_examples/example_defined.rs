pub fn main() {
    let mut x0 = 42; // Tag: 0
                     // [Unique(0)]
    let x1 = &mut x0; // Tag: 1
                      // [Unique(1), Unique(0)]
    let x2 = &mut *x1; // Tag: 2
                       // [Unique(2), Unique(1), Unique(0)]
    *x2 = 10;
    // [Unique(2), Unique(1), Unique(0)]
    *x1 = 5;
    // [Unique(1), Unique(0)]
    x0 = 3;
    // [Unique(0)]
    let x3 = &x0; // Tag: 3
                  // [SharedRO(3), Unique(0)]
    let x4 = &x0; // Tag: 4
                  // [SharedRO(4), SharedRO(3), Unique(0)]
    x0 = 5;
    // [Unique(0)]
}
