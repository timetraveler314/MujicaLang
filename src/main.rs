use crate::frontend::name_resolution::NameResolver;
use crate::util::pp::pretty_expr;

mod core;
mod backend;
mod util;
// mod examples;
mod frontend;

fn main() {
    // let input = r"
    // let fact = fun n ->
    //     if n == 0 then
    //         1
    //     else
    //         n * fact (n - 1)
    //     end
    // in
    //     fact 10
    // end
    // ";
    let input = r"let f : forall a. a -> a -> a = fun x -> fun y -> x + y in f 3 4 end";

    let ast = frontend::parse(input);
    
    println!("{:?}", ast);
    
    println!("{}", pretty_expr(&ast, 0));

    // Name Resolution
    let mut name_resolver = NameResolver::new();
    let resolved_ast = name_resolver.resolve(ast).unwrap();


}
