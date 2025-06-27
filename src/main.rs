use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use crate::backend::closure_conversion::ClosureProgram;
use crate::backend::emit_imp::emit_imp;
use crate::core::conversion::ast2knf::AST2KNF;
use crate::core::conversion::knf2anf::knf2anf;
use crate::core::conversion::monomorphization::Monomorphization;
use crate::frontend::name_resolution::NameResolver;
use crate::frontend::tyck::tyck::TypeChecker;
use crate::util::pp::pretty_expr;

mod util;
// mod examples;
mod frontend;
mod core;
mod backend;

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
    // let input = r"(2 + 3) * 4 - 5";
    // let input = r"let id = fun (x : int) -> x in id 3 end";
    let input = r"
    let id : forall a. a -> a = fun x -> x in
        let apply : forall b. (b -> b) -> b -> b = fun f x -> f x in
            let z = apply id true in
                apply id (apply id 5)
            end
        end
    end
    ";
    //
    // let input = r"
    // let const : forall a b. a -> b -> a = fun x y -> x in
    //     let x = const 1 in
    //         let y = x true in
    //             y * 2
    //         end
    //     end
    // end
    // ";
    //
    // let input = r"
    // let compose : forall a b c. (b -> c) -> (a -> b) -> a -> c =
    //   fun f g x -> f (g x) in
    //     let inc : Int -> Int = fun x -> x + 1 in
    //         let is_even : Int -> Bool = fun x -> x / 2 == 0 in
    //             compose is_even inc 7
    //         end
    //     end
    // end
    // ";

    // let input = r"let fact : int -> int = fun n -> if n == 1 then 1 else n * fact (n - 1) end in fact 10 end";

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

    // Uncurry
    let uncurried_ast = core::uncurry::uncurry(typed_ast).unwrap();

    // To KNF
    let mut ast2knf_conv = AST2KNF::new();
    let knf = ast2knf_conv.convert(uncurried_ast);
    // println!("KNF: {}", knf::pretty_expr(&knf));

    // To ANF
    let anf = knf2anf(knf).unwrap();
    println!("ANF: {}", anf.pretty());
    println!("ANF: {:?}", anf);

    // Monomorphization
    let mut mono = Monomorphization::new();
    mono.collect_instances(&anf);
    let mono_anf = mono.rewrite_expr(anf, &mut HashMap::new());

    println!("Monomorphized ANF: {}", mono_anf.pretty());

    // Closure Conversion
    let mut closure_conv = ClosureProgram::new();
    closure_conv.convert(mono_anf);
    
    println!("Closure Converted Program: {}", closure_conv.show());
    
    // IMP Emission
    let program = emit_imp(closure_conv);
    println!("Generated IMP Code:\n{}", program);

    let mut file = File::create("factorial.c").expect("Unable to create file");
    file.write_all(program.as_bytes()).expect("Unable to write to file");
    println!("Generated C code saved to factorial.c");
}
