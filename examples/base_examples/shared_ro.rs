pub fn main() {
    let mut local = 42; // h(ℓ) = (42, [Unique(0)]).
    let x = &mut local; // h(ℓ) = (42, [Unique(0), Unique(1)]).
    let shared1 = &*x;  // h(ℓ) = (42, [Unique(0), Unique(1), SharedRo(2)]).
    let shared2 = &*x;  // h(ℓ) = (42, [Unique(0), Unique(1), SharedRo(2), SharedRo(3)]).
    let val = *x;       // h(ℓ) = (42, [Unique(0), Unique(1), SharedRo(2), SharedRo(3), Unique(4)]).
    let val = *shared1; // h(ℓ) = (42, [Unique(0), Unique(1), SharedRo(2), SharedRo(3), Unique(4), Unique(5)]).
    let val = *shared2; // h(ℓ) = (42, [Unique(0), Unique(1), SharedRo(2), SharedRo(3), Unique(4), Unique(5)], Unique(6)).
    *x += 17;           // h(ℓ) = (42, [Unique(0), Unique(1)]).
    let val = *shared1; // Analysis error! The tag of shared1 is not in the stack
}