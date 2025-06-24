mod core;
mod backend;
mod util;
mod examples;
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
    let input = r"let f = fun x -> fun y -> x + y in f 3 4 end";

    let ast = frontend::parse(input);
    
    let typed_ast = frontend::hm::infer::annotate_type_var(ast, &mut frontend::hm::infer::FreshVarGenerator::new())
        .expect("Failed to annotate type variable");
    
    let mut constraints = Vec::new();
    
    frontend::hm::infer::extract_constraints(&typed_ast, &mut constraints, &mut frontend::hm::infer::MonoContext::new())
        .expect("Failed to extract constraints");
    
    // print the constraints
    for constraint in &constraints {
        println!("{:?}", constraint);
    }
    
    // Solve the constraints
    let subst = frontend::hm::infer::unify(&constraints).unwrap();
    println!("Substitution: {:?}", subst);
    
    // print the typed AST
    // println!("{:#?}", typed_ast);

    // println!("{:#?}", ast);
}
