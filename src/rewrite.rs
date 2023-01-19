mod pattern;
mod replace;

use {
    crate::{expression::*, text::*},
    pattern::*,
    replace::*,
    std::collections::VecDeque,
};

#[derive(PartialEq, Eq, Clone)]
pub struct Rule {
    pub pattern: Expression,
    pub replacement: Expression,
    pub kind: RuleKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum RuleKind {
    Replacement,
    Equality,
}

pub fn apply_rule(expr: Expression, rule: &Rule) -> Result<Expression, Expression> {
    if let Ok(bind) = match_pattern(expr.clone(), rule.pattern.clone()) {
        replace_variable(rule.replacement.clone(), &bind).map_err(|()| expr)
    } else {
        Err(expr)
    }
}

// no result wrapper in func output since you can't trust it to not do (a => a) and always return Ok
pub fn transform_recursive(
    mut expr: Expression,
    func: &mut impl FnMut(Expression) -> Expression,
) -> Expression {
    loop {
        let previous_expr = expr.clone();

        // step 1: try recursion
        if let Expression::Concrete(Concrete::Variadic(Variadic { terms, kind })) = expr {
            let mut new_terms = Vec::new();

            for old_term in terms.into_iter() {
                new_terms.push(transform_recursive(old_term, func));
            }

            expr = Variadic::new(new_terms, kind).into();
        } else if let Expression::Concrete(Concrete::Unary(Unary { argument, kind })) = expr {
            expr = Unary::new(transform_recursive(*argument, func), kind).into();
        }

        // step 2: try applying function on base
        expr = func(expr);

        // step 3: if successful on 1 or 2, loop and try again, otherwise return
        if expr == previous_expr {
            break expr;
        }
    }
}

pub trait RuleTextFormat<ErrorType>: ExprTextFormat<ErrorType> {
    fn parse_ruleset(s: &str) -> Result<Vec<Rule>, ErrorType> {
        let s = s
            .lines()
            .map(|s| {
                if let Some(i) = s.find(Self::comment()) {
                    &s[..i]
                } else {
                    s
                }
            })
            .collect::<String>();

        let mut tokens = Self::tokenize(&s);
        let mut output = Vec::new();

        while !tokens.is_empty() {
            output.push(Self::parse_tokens_rule(&mut tokens)?);
        }

        Ok(output)
    }

    fn format_rule(rule: &Rule) -> String {
        let tokens = Self::format_rule_tokens(rule);
        Self::format_tokens(tokens)
    }

    fn parse_tokens_rule(tokens: &mut VecDeque<SourceToken>) -> Result<Rule, ErrorType>;

    fn format_rule_tokens(rule: &Rule) -> VecDeque<SourceToken>;

    fn comment() -> &'static str;
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Lisp::format_rule(self))
    }
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Lisp::format_rule(self))
    }
}

#[test]
fn test_rewrite() {
    let exp = Lisp::parse("(+ 1 2)").unwrap();

    println!("expression: {exp}");

    let pattern = Lisp::parse("(+ 1 a)").unwrap();

    println!("pattern: {pattern}");

    let replacement = Lisp::parse("(func a)").unwrap();

    println!("replacement: {replacement}");

    let result = apply_rule(
        exp,
        &Rule {
            pattern,
            replacement,
            kind: RuleKind::Replacement,
        },
    );

    assert_eq!(result, Ok(Lisp::parse("(func 2)").unwrap()));

    println!("result: {result:?}");
}

#[test]
fn test_rule_parser_2() {
    let rules_string = "
        (=> (+ 1 2) 3)

        ( =>
            (+ a (+ b))
            (+ b a)
        )
    ";

    let rules = Lisp::parse_ruleset(rules_string).unwrap();

    use crate::text::*;

    assert_eq!(
        rules,
        vec![
            Rule {
                pattern: Lisp::parse("(+ 1 2)").unwrap(),
                replacement: Lisp::parse("3").unwrap(),
                kind: RuleKind::Replacement
            },
            Rule {
                pattern: Lisp::parse("(+ a (+ b))").unwrap(),
                replacement: Lisp::parse("(+ a b)").unwrap(),
                kind: RuleKind::Replacement
            }
        ]
    );

    println!("rules:");

    for rule in rules {
        println!("{}", rule);
    }
}

#[test]
fn test_recursive_transformer() {
    let exp = Lisp::parse("(* 1 (* 2 (* 3)))").unwrap();
    let rule = Lisp::parse_ruleset("(=> (* a (* b)) (* a b))")
        .unwrap()
        .pop()
        .unwrap();

    println!("rule: {rule}");
    println!("before: {exp}");

    let result = transform_recursive(exp, &mut |e| {
        apply_rule(e, &rule).unwrap_or_else(std::convert::identity)
    });

    assert_eq!(result, Lisp::parse("(* 1 2 3)").unwrap());

    println!("after: {result}");
}
