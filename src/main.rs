mod expression;
mod rewrite;
mod solve;
mod text;

use {
    crate::{expression::*, rewrite::*, solve::*, text::*},
    clap::Parser,
    std::{
        collections::BTreeMap,
        io::{prelude::*, stdin, stdout},
    },
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(clap::Subcommand, Clone)]
enum Mode {
    /// interactive shell for simplifying expressions with commands
    Shell,
    /// interactive shell for verifying identities, like Shell but automatic
    Auto,
}

fn main() {
    let args = Args::parse();

    use crate::solve::*;

    let rulesets = BTreeMap::from_iter(load_rulesets());

    match args.mode {
        Mode::Shell => expr_interactive_2(rulesets),
        Mode::Auto => identity_interactive(rulesets),
    }
}

fn expr_interactive_2(command_list: BTreeMap<String, Vec<Rule>>) {
    println!(
        "enter expression or commands:\n\n{}\n",
        command_list
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut expr = Literal::new(0).into();

    loop {
        print!("> ");
        stdout().flush().expect("error flushing output");

        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("error reading input");

        let commands: Vec<_> = command_list
            .iter()
            .filter_map(|(k, v)| if buf.contains(k) { Some(v) } else { None })
            .collect();

        if commands.is_empty() {
            match Lisp::parse(&buf) {
                Ok(new_expr) => {
                    expr = new_expr;
                    println!("{expr}");
                }
                Err(e) => {
                    println!("error parsing expression: {e:?}")
                }
            }
        } else {
            expr = ruleset_combiner(commands.as_slice())(expr);
            println!("{expr}");
        }
    }
}

fn identity_interactive(rulesets: BTreeMap<String, Vec<Rule>>) {
    use petgraph::{
        algo::{all_simple_paths, has_path_connecting},
        dot::Dot,
    };

    const MAX_SOLUTIONS_SHOW: usize = 5;
    let max_name_len = rulesets.keys().map(|s| s.len()).max().unwrap();

    println!("enter identity in form (== expr1 expr2):");

    loop {
        print!("> ");
        stdout().flush().expect("error flushing output");

        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("error reading input");

        match Lisp::parse_ruleset(&buf) {
            Ok(mut rule) => match rule.pop() {
                Some(Rule {
                    pattern,
                    replacement,
                    kind,
                }) if kind == RuleKind::Equality => {
                    let arena = elsa::FrozenIndexSet::new();
                    let graph =
                        auto_identity(pattern.clone(), replacement.clone(), &rulesets, &arena);

                    println!("process graph:\n{}", Dot::new(&graph));

                    if has_path_connecting(&graph, &pattern, &replacement, None) {
                        let mut paths: Vec<_> =
                            all_simple_paths::<Vec<_>, _>(&graph, &pattern, &replacement, 0, None)
                                .collect();

                        paths.sort_by_key(|v| v.len());

                        println!(
                            "{} solutions ({} omitted):",
                            paths.len(),
                            paths.len().saturating_sub(MAX_SOLUTIONS_SHOW)
                        );

                        for path in paths.into_iter().take(MAX_SOLUTIONS_SHOW).rev() {
                            println!("\n{} steps:", path.len());

                            println!(" {:>max_name_len$} | {pattern}", "start");

                            for pair in path.as_slice().windows(2) {
                                let [before, after] = pair else {unreachable!()};
                                println!(
                                    " {:>max_name_len$} | {after}",
                                    graph.edge_weight(before, after).unwrap()
                                );
                            }
                        }
                    } else {
                        println!("couldn't find any solutions");
                    }
                }
                Some(_) => println!("error: wrong type of rule"),
                None => println!("error: empty rule"),
            },
            Err(e) => println!("error parsing rule: {e:?}"),
        }
    }
}
