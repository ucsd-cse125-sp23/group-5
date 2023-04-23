
use common::add;
use client::run;
fn main() {
    println!("Client: Hello, world!");
    
    // example of using a function defined in the common crate
    let result = add(2, 2);
    println!("Result: {}", result);

    pollster::block_on(run());
}
