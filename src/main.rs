mod stack;

use stack::*;
fn main() {
    println!("Hello, world!");
    let mut stack: Stack<isize> = Stack::new();
    stack.push(1);
    let item = stack.pop();
    assert_eq!(item.unwrap(), 1);
}
