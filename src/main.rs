mod core;
mod backend;
mod util;
mod examples;
mod frontend;

fn main() {
    let input = r"
    let fun fact (n: int) : int =
        if n == 0 then
            1
        else
            n * fact (n - 1)
        end
    in
        fact 10
    end
    ";
    let ast = frontend::parse(input);
    let typed_ast = ast.to_typed().unwrap();
    let knf = core::conversion::ast2knf::ast2knf(typed_ast);
    let anf = core::conversion::knf2anf::knf2anf(knf).unwrap();
    let closure = anf.anf2closure().unwrap();
    let result = backend::emit_imp::emit_imp(closure);

    let mut file = std::fs::File::create("examples/whole_fact.c").expect("Unable to create file");
    std::io::Write::write_all(&mut file, result.as_bytes()).expect("Unable to write data");
}
