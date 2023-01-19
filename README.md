# Identity (CAS)

This program can verify some simple trigonometric identities that involve arithmetic.

## How to run

1. Install the [Rust](https://www.rust-lang.org/) tool chain
2. Run the command: `cargo run -- <arguments here>`

## Math syntax

For now, all expressions are expected to be written in a [Lisp](https://en.wikipedia.org/wiki/Lisp_\(programming_language\)) like notation, and non-commutative operations are not allowed (this was mostly to simplify parsing and the AST). Extra whitespace is ignored.

This means expressions like `1 + 2` are written like `(+ 1 2)`, and `a * b * c` becomes `(* a b c)`.

Also, subtraction and division are only represented as negation `-` and reciprocal `/` and only take 1 argument, so `3 - 4` becomes `(+ 3 (- 4))` and `5 / 6` becomes `(* 5 (/ 6))`.

Any non-numerical string in the first position in a `()` represents a custom function name, and they are limited to 1 argument here for simplicity, like `sin` in `(sin 5)`. Otherwise, those strings represent variables, like `a` in `(cos a)`.

Rewrite `=>` and equality `==` take 2 arguments, but they are only supported as the top level operator in a rule or an identity. In a rewrite rule, the first argument is the pattern and the second argument is replacement, and variables serve as binding sites.

## Run modes

There are two ways to run the executable

1. `cargo run -- shell`
    Enter plain expressions and rewriting commands

2. `cargo run -- auto`
    Enter identities composed of two expressions under the `==` operator

The first lets you manually rewrite the expression using the provided sets of rules, and the second does it automatically.

For each identity, the second outputs the search graph in a form you can render with [GraphViz](https://graphviz.org/) and a few of the shortest paths labeled with each step, which represent the solutions, if any.
