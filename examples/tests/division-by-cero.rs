
fn main() {
    let n = 0;
    let a = 100;

    let b = a / n; // Error: division by zero!

    if n != 0 {
        let c = a / n; // OK
    }
}