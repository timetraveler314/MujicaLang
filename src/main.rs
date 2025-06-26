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
            apply id ()
        end
    end
    ";
    
    let input = r"
    let const : forall a b. a -> b -> a = fun x y -> x in
        let x = const 1 in
            let y = x true in
                y * 2
            end
        end
    end
    ";
    
    let input = r"
    let compose : forall a b c. (b -> c) -> (a -> b) -> a -> c = 
      fun f g x -> f (g x) in
        let inc : Int -> Int = fun x -> x + 1 in
            let is_even : Int -> Bool = fun x -> x / 2 == 0 in
                compose is_even inc 7
            end
        end
    end
    ";

    let ast = frontend::parse(input);

    println!("Parsed: \n{}", pretty_expr(&ast, 0));

    // Name Resolution
    let mut name_resolver = NameResolver::new();
    let resolved_ast = name_resolver.resolve(ast).unwrap();

    // Tyck
    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.tyck(resolved_ast).unwrap();
    
    println!("Resolved AST: {}", pretty_expr(&typed_ast, 0));
    println!("{}", type_checker);
}
