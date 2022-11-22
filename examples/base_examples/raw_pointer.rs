fn main() {
    let mut local = 5; // Stored at location ℓ, and with tag 0, h(ℓ) = (5, [Unique(0)]).
    let raw_pointer = &mut local as *mut i32; // = Pointer(ℓ, ⊥)
                                              // The temporary reference gets tag 1, and gets pushed: h(ℓ) = (5, [Unique(0), Unique(1)]).
                                              // Then the raw pointer gets pushed: h(ℓ) = (5, [Unique(0), Unique(1), SharedRW]).
                                              // (new-mutable-ref, new-mutable-raw-1)
    let result = unsafe {
        example1(
            &mut *raw_pointer, // = Pointer(ℓ, 2)
            // First reference gets added on top of the raw pointer: h(ℓ) = (5, [. . . , SharedRW, Unique(2)]).
            // This uses the raw pointer! (new-mutable-ref)
            &mut *raw_pointer, // = Pointer(ℓ, 3)
                               // Using raw_pointer here pops the first reference off the stack: h(ℓ) = (5, [. . . , SharedRW, Unique(3)]).
                               // This uses the raw pointer! (new-mutable-ref)
        )
    }; // Next: jump to example1 (line 17).
    //println!(" {} ", result); // Prints "13".
}
fn example1(x: &mut i32, y: &mut i32) -> i32 {
    // x = Pointer(ℓ, 2), y = Pointer(ℓ, 3), h(ℓ) = (5, [Unique(0), Unique(1), SharedRW, Unique(3)])
    *x = 42;
    // Analysis error! Tag of x (which is 2) is not in the stack. Program has undefined behavior.
    *y = 13;
    return *x; // We want to optimize this to return the constant 42.
}
