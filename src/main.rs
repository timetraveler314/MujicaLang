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
    
    println!("{:#?}", ast);
}
