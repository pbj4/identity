use {crate::expression::*, std::collections::BTreeMap};

// pattern variables treated as capture groups
// all expressions MUST be completely bound to structures, literals, or variables

pub fn match_pattern(
    expr: Expression,
    patt: Expression,
) -> Result<BTreeMap<Variable, Expression>, ()> {
    match_pattern_rec(vec![MatchGroup::Single { expr, patt }], BTreeMap::new())
}

// ONLY use recursion for backtracking, do something else for tree traversal
// ONLY escape out of loop with return Err or return recursion()

fn match_pattern_rec(
    mut stack: Vec<MatchGroup>,
    mut bind: BTreeMap<Variable, Expression>,
) -> Result<BTreeMap<Variable, Expression>, ()> {
    while let Some(match_comb) = stack.pop() {
        match match_comb {
            MatchGroup::Single { expr, patt } => match (expr, patt) {
                (Expression::Concrete(ec), Expression::Concrete(pc)) => match (ec, pc) {
                    (
                        Concrete::Variadic(Variadic {
                            terms: eterms,
                            kind: ekind,
                        }),
                        Concrete::Variadic(Variadic {
                            terms: pterms,
                            kind: pkind,
                        }),
                    ) if ekind == pkind => {
                        stack.push(MatchGroup::Multiple {
                            expr_list: eterms.into_iter().collect(),
                            patt_list: pterms.into_iter().collect(),
                            kind: ekind,
                        });
                        continue; // operate on next
                    }
                    (
                        Concrete::Unary(Unary {
                            argument: earg,
                            kind: ekind,
                        }),
                        Concrete::Unary(Unary {
                            argument: parg,
                            kind: pkind,
                        }),
                    ) if ekind == pkind => {
                        stack.push(MatchGroup::Single {
                            expr: *earg,
                            patt: *parg,
                        });
                        continue; // operate on next
                    }
                    (Concrete::Literal(el), Concrete::Literal(pl)) if el == pl => continue, // good match, operate on next
                    _ => return Err(()), // mismatching literal
                },
                (expr, Expression::Variable(pv)) => {
                    if *bind.entry(pv).or_insert_with(|| expr.clone()) != expr {
                        return Err(());
                    } else {
                        continue; // good match, operate on next
                    }
                }

                _ => return Err(()), // mismatching expression
            },
            MatchGroup::Multiple {
                expr_list,
                patt_list: mut patt_rest,
                kind,
            } => {
                if let Some(patt) = patt_rest.pop() {
                    let esize = match expr_list.len() {
                        0 => return Err(()), // empty expression list
                        n => n,
                    };
                    let is_var = matches!(patt, Expression::Variable(_));

                    for bitmask in (0..).map_while(|n| {
                        if is_var {
                            all_comb(n, esize)
                        } else {
                            set_one(n, esize)
                        }
                    }) {
                        let mut bitmask = bitmask.into_iter();
                        let (mut expr, expr_rest): (Vec<_>, Vec<_>) = expr_list
                            .iter()
                            .cloned()
                            .partition(|_| bitmask.next().unwrap());

                        let expr = if expr.len() == 1 {
                            expr.pop().unwrap()
                        } else {
                            Variadic::new(expr, kind).into()
                        };

                        if let Ok(bind) = match_pattern_rec(
                            stack
                                .iter()
                                .cloned()
                                .chain([
                                    MatchGroup::Multiple {
                                        expr_list: expr_rest,
                                        patt_list: patt_rest.clone(),
                                        kind,
                                    },
                                    MatchGroup::Single {
                                        expr,
                                        patt: patt.clone(),
                                    },
                                ])
                                .collect(),
                            bind.clone(),
                        ) {
                            // should only return if base case is reached
                            return Ok(bind);
                        }
                    }

                    // no combination of matches work
                    return Err(());
                } else if expr_list.is_empty() {
                    // both empty, operate on next
                    continue;
                } else {
                    // empty pattern list
                    return Err(());
                }

                //unreachable!("must diverge");
            }
        }

        //unreachable!("must diverge");
    }

    // base case: backtrack stack empty
    Ok(bind)
}

#[derive(Clone, Debug)]
enum MatchGroup {
    Single {
        expr: Expression,
        patt: Expression,
    },
    Multiple {
        expr_list: Vec<Expression>,
        patt_list: Vec<Expression>,
        kind: VariadicKind,
    },
}

// transforms n in 0..size to bitmask of length size
fn set_one(n: usize, size: usize) -> Option<Vec<bool>> {
    if n < size {
        Some((0..size).map(|i| n == i).collect())
    } else {
        None
    }
}

// transforms n in 0..2^size to bitmask of length size
fn all_comb(n: usize, size: usize) -> Option<Vec<bool>> {
    let max = 2usize.pow(size as u32);
    if n < max {
        Some((0..size).map(|i| (n >> i) & 1 == 0).collect())
    } else {
        None
    }
}

#[test]
fn basic_test() {
    use crate::text::*;

    let pat = Lisp::parse("(func2 (/ (- var0)))").unwrap();
    let exp = Lisp::parse("(func2 (/ (- (* 2 2))))").unwrap();

    let results = match_pattern(exp, pat.clone());

    assert_eq!(
        results,
        Ok(BTreeMap::from([(
            Variable::new("var0"),
            Lisp::parse("(* 2 2)").unwrap(),
        )]))
    );

    println!("match: {:?}", results);

    let exp = Lisp::parse("(+ 3 1)").unwrap();

    let results = match_pattern(exp, pat);

    assert_eq!(results, Err(()));

    println!("match: {results:?}");
}

#[test]
fn basic_variadic_test() {
    use crate::text::*;

    let pat = Lisp::parse("(* (+ a 4))").unwrap();

    println!("pattern: {pat}");

    let exp = Lisp::parse("(* (+ 4 10))").unwrap();

    println!("expression: {exp}");

    let results = match_pattern(exp, pat.clone());

    assert_eq!(
        results,
        Ok(BTreeMap::from([(
            Variable::new("a"),
            Lisp::parse("10").unwrap(),
        )]))
    );

    println!("match: {results:?}");

    let exp = Lisp::parse("(* (+ 4))").unwrap();

    let results = match_pattern(exp, pat);

    assert_eq!(
        results,
        Ok(BTreeMap::from([(
            Variable::new("a"),
            Lisp::parse("(+)").unwrap()
        )]))
    );

    println!("match: {results:?}");
}

#[test]
fn advanced_variadic_test() {
    use crate::text::*;

    let pat = Lisp::parse("(+ 1 a)").unwrap();

    println!("pattern: {pat}");

    let exp = Lisp::parse("(+ 1 2 3 4 (* var7 var8))").unwrap();

    println!("expression: {exp}");

    let results = match_pattern(exp, pat);

    assert_eq!(
        results,
        Ok(BTreeMap::from([(
            Variable::new("a"),
            Lisp::parse("(+ 2 3 4 (* var7 var8))").unwrap(),
        )]))
    );

    println!("match: {results:?}");
}
