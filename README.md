# MujicaLang

MujicaLang (ML) is a simple functional programming language that mimics the syntax
and semantics of a subset of ML-like languages. It is a course project for
the course _Compiler Principles (Honor Track)_.

## Overview

MujicaLang demonstrates various compiler phases, currently focusing on:

- ANF (A-Normal Form) representation
- Closure conversion
- Intermediate language (IMP, which is currently a strict subset of C) emission

## Project Structure

```
MujicaLang/
├── src/
│   ├── core/         - Core language types and ANF representation
│   ├── backend/      - Transformation passes and code generation
│   ├── util/         - Utility functions and formatters
│   └── examples/     - Example programs and test cases
```

## Stages

- **ANF Conversion**: Convert the source code into A-normal form.
  - ANF is a representation where all intermediate results are bound to variables, and in addition to the K-normal form restrictions,
    ANF further restricts that `let` statements can only be nested in the body part, not in the assignment part.
  - BNF grammar for ANF:
    ```
    atom ::= intconst(i32) | var(ident)
    # specifically, we allow `if` to be in the position to avoid exopnential grow of the program
    c-expr ::= atom | op(atom, ...) | call(atom, atom, ...) | if(atom, expr, expr) 
    expr ::= c-expr | let(ident, c-expr, expr) | letfun(ident, ident, expr, expr)
    ```
- **Closure Conversion**: Transform from A-normal form to closure form, by lambda lifting and
  converting free variables into closure captures.
  - The BNF is similar to the one above, but with the following changes:
    ```
    ...
    expr ::= c-expr | let(ident, c-expr, expr) | letclos(ident, closure, expr)
    ```
- **IMP Generation**: Emit IMP code from the closure form, using compiler-defined `struct`'s
  for passing and calling closures.

## Examples

The `examples` directory contains sample programs that demonstrate the language's features:

- `factorial`: Recursive factorial calculation
- `closure_return`: Example of returning closures from functions
- `simple_1`: Basic arithmetic and variable binding
