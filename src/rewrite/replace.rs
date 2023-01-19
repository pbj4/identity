use {crate::expression::*, std::collections::BTreeMap};

pub fn replace_variable(
    expr: Expression,
    bind: &BTreeMap<Variable, Expression>,
) -> Result<Expression, ()> {
    match expr {
        Expression::Concrete(c) => match c {
            Concrete::Variadic(Variadic { terms, kind }) => {
                let mut terms_new = Vec::new();

                for term in terms.into_iter() {
                    match replace_variable(term, bind)? {
                        Expression::Concrete(Concrete::Variadic(Variadic {
                            terms: inner_terms,
                            kind: inner_kind,
                        })) if inner_kind == kind || inner_terms.is_empty() => {
                            terms_new.extend(inner_terms.into_iter())
                        } // flatten variadics, empty ones are removed
                        term => terms_new.push(term),
                    }
                }

                Ok(Variadic::new(terms_new, kind).into())
            }
            Concrete::Unary(Unary { argument, kind }) => {
                Ok(Unary::new(replace_variable(*argument, bind)?, kind).into())
            }
            Concrete::Literal(Literal { value }) => Ok(Literal::new(value).into()),
        },
        Expression::Variable(v) => Ok(bind.get(&v).cloned().ok_or(())?),
    }
}

#[test]
fn test_replace() {
    use crate::text::*;

    let exp = Lisp::parse("(+ var9 (* var7 (+ var5 var9)))").unwrap();

    println!("expression: {exp}");

    let bind = BTreeMap::from([
        (Variable::new("var5"), Lisp::parse("42").unwrap()),
        (Variable::new("var7"), Lisp::parse("65").unwrap()),
        (Variable::new("var9"), Lisp::parse("3").unwrap()),
    ]);

    println!("bindings: {bind:?}");

    let result = replace_variable(exp, &bind);

    assert_eq!(result, Ok(Lisp::parse("(+ 3 (* 65 (+ 42 3)))").unwrap()));
    println!("result: {result:?}");
}
