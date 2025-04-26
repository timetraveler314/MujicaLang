mod core;
mod backend;
mod util;
mod examples;

fn main() {
    let program = crate::examples::ast::fun_fact();
    let mut file = std::fs::File::create("examples/whole_fact.c").expect("Unable to create file");
    std::io::Write::write_all(&mut file, program.as_bytes()).expect("Unable to write data");
}
