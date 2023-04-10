use common::add;

fn main() {
    println!("Server: Hello, world!");

    // example of using a function defined in the common crate
    let result = add(3, 4);
    println!("Result: {}", result);
}
