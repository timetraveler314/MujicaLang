use crate::frontend::name_resolution::NameResolver;
use crate::frontend::tyck::tyck::TypeChecker;
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
    // let input = r"let f : forall a. a -> a -> a = fun x -> fun y -> x + y in f 3 4 end";
    let input = r"let id : forall a. a -> a = fun x -> x in id 3 end";
    // let input = r"let id = fun (x : int) -> x in id 3 end";
    let input = r"
    let id : forall a. a -> a = fun x -> x in
        let apply : forall b. (b -> b) -> b -> b = fun f x -> f x in
            apply id 42
        end
    end
    ";

    let ast = frontend::parse(input);

    println!("Parsed: \n{}", pretty_expr(&ast, 0));

    // Name Resolution
    let mut name_resolver = NameResolver::new();
    let mut resolved_ast = name_resolver.resolve(ast).unwrap();

    // Tyck
    let mut type_checker = TypeChecker::new();
    type_checker.infer(&mut resolved_ast).unwrap();
    type_checker.final_apply(&mut resolved_ast);
    
    println!("Resolved AST: {}", pretty_expr(&resolved_ast, 0));
    println!("{}", type_checker);
}
