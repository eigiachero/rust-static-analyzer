fn example(arg: &mut u32) -> u32 {
    let local = arg;
    *local = 15;
    1
}

fn main() {
    let mut id = 5;
    let arg = &mut id;
    let _pointer = &arg;
    let _result = example(arg);
}