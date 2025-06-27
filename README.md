# MujicaLang Compiler

A functional language compiler built in Rust, targeting C as the backend. MujicaLang is an ML-style expression-oriented language featuring lexical scoping, rank-1 parametric polymorphism, and type inference. This project is developed as part of a course project for *Compiler Principles (Honor Track)* in Spring 2025 and demonstrates a full-stack pipeline from parsing to code generation.

## 🚀 Quick Start

Compile and run a MujicaLang program:

```bash
cargo run --release -- examples/fact.ml --output examples/fact.c --compile --exec examples/fact
./examples/fact # Output: 3628800
```

## Project Overview

The compiler takes a MujicaLang source file through a series of transformation stages before emitting C code:

MujicaLang Source

[Frontend]
- LALRPOP Parser → Curried AST
- Name Resolution → Resolved AST
- Type Checking/Inference → Typed AST

[Middle-end]
- Uncurrying
- K-Normal Form (KNF)
- A-Normal Form (ANF)
- Monomorphization
- Closure Conversion

[Backend]
- C Emission
- Executable C Code

## Language Design

MujicaLang is expression-based and supports:

- First-class functions and lambdas
- `let` bindings (recursive when type-annotated)
- Arithmetic and boolean operations
- Conditionals (`if ... then ... else ... end`)
- Rank-1 Parametric polymorphism (via `forall`)

Example:

```ml
let id : forall a. a -> a = fun x -> x in
  let apply : forall b. (b -> b) -> b -> b = fun f x -> f x in
    apply id 5
  end
end
```

## Type System

MujicaLang uses Bidirectional Type Checking rather than Algorithm W:

- Mandatory annotations on lambda parameters.
- Polymorphism only via explicit forall in let bindings.
- Check/infer split:
  - ```infer(expr: &mut ResolvedASTExpr) -> Result<Ty, FrontendError>```
  - ```check(expr: &mut ResolvedASTExpr, expected_ty: Ty) -> Result<(), FrontendError>```

## Compiler Internals

- Frontend (`src/frontend`): Parses and type-checks the source code.
  - `tyck/`: Bidirectional type checker.
  - `hm/`: Legacy Hindley-Milner checker.
- Core (`src/core`): Intermediate representations and conversion passes.
- Backend (`src/backend`): Closure conversion and C code generation.
- Utilities (`src/util`): Name generation, pretty-printing, etc.
- Intermediate Representations (IR)
  - K-Normal Form (KNF): Flattens expressions to simplify order of evaluation.
  - A-Normal Form (ANF): Eliminates nested lets, producing C-like imperative structure.
  - Closure Form: Captures lexical environments into heap-allocated closures for C.