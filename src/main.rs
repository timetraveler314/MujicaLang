use crate::core::conversion::knf2anf;

mod core;
mod backend;
mod util;
mod examples;

fn main() {
    let converted1 = knf2anf::knf2anf(examples::knf::get_simple()).unwrap();
    println!("Converted 1 ANF: {}", converted1);
    
    // examples::anf::compile("factorial");
    // examples::anf::compile("closure_return");
    // examples::anf::compile("simple_1");
}
