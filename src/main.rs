mod core;
mod backend;
mod util;
mod examples;

fn main() {
    examples::anf::compile("factorial");
    examples::anf::compile("closure_return");
    examples::anf::compile("simple_1");
}
