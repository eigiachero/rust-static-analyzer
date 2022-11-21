pub fn main(){
    let mut local = 42; // Stored at location ℓ, and with tag 0.
    // The initial stack: h(ℓ) = (42, [Unique(0)])
    let x = & mut local ; // = Pointer(ℓ, 1)
    // Use local, push tag of x (1) on the stack:
    // h(ℓ) = (42, [Unique(0), Unique(1)]). (new-mutable-ref)
    let y = & mut *x; // = Pointer(ℓ, 2)
    // Use x, push tag of y (2):
    //h(ℓ) = (42, [Unique(0), Unique(1), Unique(2)]). (new-mutable-ref)
    *x += 1;
    // Remove tag of y (2) to bring tag of x (1) to the top:
    // h(ℓ) = (43, [Unique(0), Unique(1)]). (use-1)
    *y = 2;
    // Undefined behavior! Stack principle violated: tag of y (2) is not in the stack. (use-1)
}
