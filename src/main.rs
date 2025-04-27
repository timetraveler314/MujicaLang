mod core;
mod backend;
mod util;
mod examples;
mod frontend;

fn main() {
    let program = crate::examples::ast::fun_fact();
    let mut file = std::fs::File::create("examples/whole_fact.c").expect("Unable to create file");
    std::io::Write::write_all(&mut file, program.as_bytes()).expect("Unable to write data");

    let result = frontend::parse("let x = 1 in let y = 2 in f x z z (f y) + y end end");
    println!("{:?}", result);
}
