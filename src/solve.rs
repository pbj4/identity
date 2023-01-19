use {
    crate::{expression::*, rewrite::*, text::*},
    petgraph::prelude::*,
    std::{collections::BTreeMap, path::PathBuf},
};

const RULESETS_DIR: &str = "./rules";
const MACRO_PREFIX: &str = "#";

pub fn load_rulesets() -> Vec<(String, Vec<Rule>)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(RULESETS_DIR);

    let (mut rules, macros): (BTreeMap<_, _>, BTreeMap<_, _>) = extract_rules_macros(
        path.read_dir()
            .unwrap_or_else(|e| panic!("error loading ruleset directory: {path:?}, {e}"))
            .map(|d| d.unwrap().path()),
    )
    .map(|(f, r, m)| ((f.clone(), r), (f, m)))
    .unzip();

    for (file, macros) in macros.into_iter() {
        for m in macros {
            if let [command, args @ ..] = m
                .trim()
                .strip_prefix(MACRO_PREFIX)
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>()
                .as_slice()
            {
                match *command {
                    "map" => {
                        let f = ruleset_combiner(
                            &args
                                .iter()
                                .map(|s| {
                                    rules
                                        .get(*s)
                                        .unwrap_or_else(|| panic!("nonexistent ruleset: {s}"))
                                })
                                .collect::<Vec<_>>(),
                        );

                        let (file, ruleset) = rules.remove_entry(&file).unwrap();
                        let ruleset = ruleset
                            .into_iter()
                            .map(
                                |Rule {
                                     pattern,
                                     replacement,
                                     kind,
                                 }| Rule {
                                    pattern: f(pattern),
                                    replacement: f(replacement),
                                    kind,
                                },
                            )
                            .collect();

                        rules.insert(file, ruleset);
                    }
                    u => panic!("unknown macro: {u}"),
                }
            } else {
                panic!("empty macro in: {file}");
            }
        }
    }

    rules.into_iter().collect()
}

fn extract_rules_macros(
    iter: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = (String, Vec<Rule>, Vec<String>)> {
    iter.map(|file| {
        let stem = file.file_stem().unwrap().to_str().unwrap().to_string();

        let ext = file
            .extension()
            .unwrap_or_else(|| panic!("file missing extension: {file:?}"))
            .to_str()
            .unwrap()
            .to_owned();
        let name = file.file_name().unwrap().to_owned();

        let text = std::fs::read_to_string(file).unwrap();
        let (macros, text): (Vec<_>, Vec<_>) =
            text.lines().partition(|l| l.starts_with(MACRO_PREFIX));
        let macros: Vec<_> = macros.into_iter().map(|s| s.to_string()).collect();
        let text = text.into_iter().collect::<Vec<_>>().as_slice().join("\n");

        let rules = match ext.as_str() {
            "lisp" => Lisp::parse_ruleset(&text),
            u => panic!("unknown file extension: {u:?}"),
        }
        .unwrap_or_else(|e| panic!("error parsing file: {name:?}, {e:?}"));

        (stem, rules, macros)
    })
}

pub fn ruleset_combiner(rule_sources: &[&Vec<Rule>]) -> impl Fn(Expression) -> Expression {
    let rules: Vec<_> = rule_sources.iter().copied().flatten().cloned().collect();
    move |expr| apply_ruleset(expr, &rules)
}

fn apply_ruleset(mut expr: Expression, ruleset: &[Rule]) -> Expression {
    loop {
        let previous_expr = expr.clone();

        for rule in ruleset.iter() {
            expr = transform_recursive(expr, &mut |e| {
                apply_rule(e, rule).unwrap_or_else(std::convert::identity)
            });
        }

        if expr == previous_expr {
            break expr;
        }
    }
}

pub fn auto_identity<'a>(
    left: Expression,
    right: Expression,
    rulesets: &BTreeMap<String, Vec<Rule>>,
    arena: &'a elsa::index_set::FrozenIndexSet<Box<Expression>>,
) -> GraphMap<&'a Expression, String, Directed> {
    let mut graph: GraphMap<&Expression, String, Directed> = GraphMap::new();
    let mut unprocessed = vec![left, right];

    let always_apply = rulesets.get("simplify").unwrap();

    while let Some(expr) = unprocessed.pop() {
        for (name, ruleset) in rulesets.iter() {
            let expr_new = ruleset_combiner(&[ruleset, always_apply])(expr.clone());

            if expr_new != expr {
                let new_ref = arena.insert(Box::new(expr_new.clone()));

                if !graph.contains_node(new_ref) {
                    unprocessed.push(expr_new);
                }

                graph.add_edge(
                    arena.insert(Box::new(expr.clone())),
                    new_ref,
                    name.to_string(),
                );
            }
        }
    }

    graph
}

#[test]
fn test_ruleset_loading() {
    println!("rules: ");
    for (name, rules) in load_rulesets() {
        println!("{name}");
        for rule in rules {
            println!("{rule}");
        }
    }
}
