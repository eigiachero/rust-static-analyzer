pub fn main() {
    let x = &mut 1u8; // tag: `Uniq(0)`
                      // stack: [Uniq(0)]

    let y = &mut *x; // tag: `Uniq(1)`
                     // stack: [Uniq(0), Uniq(1)]

    // Pop until `Uniq(1)`, the tag of `y`, is on top of the stack:
    // Nothing changes.
    *y = 5;
    // stack: [Uniq(0), Uniq(1)]

    // Pop until `Uniq(0)`, the tag of `x`, is on top of the stack:
    // We pop `Uniq(1)`.
    *x = 3;
    // stack: [Uniq(0)]

    // Pop until `Uniq(1)`, the tag of `y`, is on top of the stack:
    // That is not possible, hence we have undefined behavior.
    let _val = *y;
}
