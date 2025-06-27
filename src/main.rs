use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use clap::Parser;
use crate::backend::closure_conversion::ClosureProgram;
use crate::backend::emit_imp::emit_imp;
use crate::core::conversion::ast2knf::AST2KNF;
use crate::core::conversion::knf2anf::knf2anf;
use crate::core::conversion::monomorphization::Monomorphization;
use crate::frontend::name_resolution::NameResolver;
use crate::frontend::tyck::tyck::TypeChecker;
use crate::util::pp::pretty_expr;

mod util;
// mod old_examples;
mod frontend;
mod core;
mod backend;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Source file to compile
    input: PathBuf,

    /// Output C file
    #[arg(short, long, default_value = "output.c")]
    output: PathBuf,

    /// Compile with GCC
    #[arg(short, long)]
    compile: bool,

    /// Output executable name (only if --compile is set)
    #[arg(short, long, default_value = "a.out")]
    exec: String,
}

fn main() {
    let args = Args::parse();

    let input_code = std::fs::read_to_string(&args.input)
        .expect("Failed to read input source file");

    let c_code = compile_to_c(&input_code);

    std::fs::write(&args.output, c_code).expect("Failed to write output C file");
    println!("Generated C code saved to {}", args.output.display());

    if args.compile {
        use std::process::Command;

        let status = Command::new("gcc")
            .arg(&args.output)
            .arg("-o")
            .arg(&args.exec)
            .status()
            .expect("Failed to invoke GCC");

        if status.success() {
            println!("Compilation succeeded. Executable: {}", args.exec);
        } else {
            eprintln!("GCC failed with exit code: {}", status);
        }
    }
}

fn compile_to_c(input_code: &str) -> String {
    use crate::backend::closure_conversion::ClosureProgram;
    use crate::backend::emit_imp::emit_imp;
    use crate::core::conversion::ast2knf::AST2KNF;
    use crate::core::conversion::knf2anf::knf2anf;
    use crate::core::conversion::monomorphization::Monomorphization;
    use crate::frontend::name_resolution::NameResolver;
    use crate::frontend::tyck::tyck::TypeChecker;

    let ast = frontend::parse(input_code);
    let mut name_resolver = NameResolver::new();
    let resolved_ast = name_resolver.resolve(ast).unwrap();

    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.tyck(resolved_ast).unwrap();

    let uncurried_ast = core::uncurry::uncurry(typed_ast).unwrap();

    let mut ast2knf_conv = AST2KNF::new();
    let knf = ast2knf_conv.convert(uncurried_ast);

    let anf = knf2anf(knf).unwrap();

    let mut mono = Monomorphization::new();
    mono.collect_instances(&anf);
    let mono_anf = mono.rewrite_expr(anf, &mut std::collections::HashMap::new());

    let mut closure_conv = ClosureProgram::new();
    closure_conv.convert(mono_anf);

    emit_imp(closure_conv)
}