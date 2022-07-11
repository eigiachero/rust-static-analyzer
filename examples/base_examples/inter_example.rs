#[derive(Debug)]
struct MyType {id: i32}

fn example(mut arg: MyType) -> i32 {
    let local = &mut arg;
    local.id = 15;
    1
}

fn main() {
    let id = 5;
    let arg = MyType {id};
    let _pointer = &arg;
    let _result = example(arg);
}