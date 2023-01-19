pub use lisp::*;
use {
    crate::{expression::*, rewrite::*},
    std::collections::{BTreeMap, VecDeque},
};

mod lisp;

const LGROUP: &str = "(";
const RGROUP: &str = ")";
const VARIADIC_MAP: [(&str, VariadicKind); 2] = [
    ("+", VariadicKind::Addition),
    ("*", VariadicKind::Multiplication),
];
const UNARY_MAP: [(&str, UnaryKind); 2] =
    [("-", UnaryKind::Negation), ("/", UnaryKind::Reciprocal)];
const RULE_MAP: [(&str, RuleKind); 2] = [("=>", RuleKind::Replacement), ("==", RuleKind::Equality)];

#[derive(Debug, PartialEq)]
pub enum SourceToken {
    Literal(LiteralValue),
    VariadicOperator(VariadicKind),
    UnaryOperator(UnaryKind),
    RuleOperator(RuleKind),
    Text(String),
    LGroup,
    RGroup,
}

pub trait ExprTextFormat<ErrorType> {
    fn parse(s: &str) -> Result<Expression, ErrorType> {
        let mut tokens = Self::tokenize(s);
        Self::parse_tokens(&mut tokens)
    }

    fn format(expr: &Expression) -> String {
        let tokens = Self::format_expr(expr);
        Self::format_tokens(tokens)
    }

    fn tokenize(s: &str) -> VecDeque<SourceToken> {
        let delim = |c| LGROUP.contains(c) || RGROUP.contains(c);
        let variadic_map = BTreeMap::from(VARIADIC_MAP);
        let unary_map = BTreeMap::from(UNARY_MAP);
        let rule_map = BTreeMap::from(RULE_MAP);

        s.split_inclusive(delim)
            .flat_map(move |s| {
                if let Some(i) = s.rfind(delim) {
                    if i > 0 {
                        let (a, b) = s.split_at(i);
                        return vec![a, b];
                    }
                }
                vec![s]
            })
            .flat_map(|s| s.split_whitespace())
            .map(move |s| {
                match s {
                    LGROUP => return SourceToken::LGroup,
                    RGROUP => return SourceToken::RGroup,
                    _ => {}
                }

                if let Some(kind) = rule_map.get(s) {
                    return SourceToken::RuleOperator(*kind);
                }

                if let Some(kind) = variadic_map.get(s) {
                    return SourceToken::VariadicOperator(*kind);
                }

                if let Some(kind) = unary_map.get(s) {
                    return SourceToken::UnaryOperator(kind.clone());
                }

                if let Ok(n) = s.parse::<i32>() {
                    return SourceToken::Literal(LiteralValue::Integer(n));
                }

                if s.chars().all(|c| c.is_uppercase()) {
                    return SourceToken::Literal(LiteralValue::Constant(s.to_string()));
                }

                SourceToken::Text(s.to_string())
            })
            .collect()
    }

    fn format_tokens(mut tokens: VecDeque<SourceToken>) -> String {
        let variadic_map = BTreeMap::from_iter(VARIADIC_MAP.iter().copied().map(|(a, b)| (b, a)));
        let unary_map = BTreeMap::from_iter(UNARY_MAP.iter().map(|(a, b)| (b.clone(), *a)));
        let rule_map = BTreeMap::from_iter(RULE_MAP.iter().map(|(a, b)| (b, a)));

        let mut output = String::new();

        while let Some(token) = tokens.pop_front() {
            output.push_str(&match token {
                SourceToken::Literal(l) => match l {
                    LiteralValue::Integer(i) => i.to_string(),
                    LiteralValue::Constant(c) => c,
                },
                SourceToken::VariadicOperator(v) => variadic_map.get(&v).unwrap().to_string(),
                SourceToken::UnaryOperator(u) => {
                    if let Some(s) = unary_map.get(&u) {
                        s.to_string()
                    } else {
                        let UnaryKind::Named { id } = u else {unreachable!()};
                        id.as_ref().to_string()
                    }
                }
                SourceToken::LGroup => LGROUP.to_string(),
                SourceToken::RGroup => RGROUP.to_string(),
                SourceToken::RuleOperator(r) => rule_map.get(&r).unwrap().to_string(),
                SourceToken::Text(s) => s,
            });
        }

        output
    }

    fn parse_tokens(tokens: &mut VecDeque<SourceToken>) -> Result<Expression, ErrorType>;

    fn format_expr(expr: &Expression) -> VecDeque<SourceToken>;
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Lisp::format(self))
    }
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Lisp::format(self))
    }
}

#[test]
fn test_tokenizer() {
    use SourceToken::*;

    let tokens: Vec<_> = Lisp::tokenize("(+ 1 ( * )((a (sin 4) (cos 2)))")
        .into_iter()
        .collect();

    assert_eq!(
        vec![
            LGroup,
            VariadicOperator(VariadicKind::Addition),
            Literal(LiteralValue::Integer(1)),
            LGroup,
            VariadicOperator(VariadicKind::Multiplication),
            RGroup,
            LGroup,
            LGroup,
            Text("a".to_string()),
            LGroup,
            Text("sin".to_string()),
            Literal(LiteralValue::Integer(4)),
            RGroup,
            LGroup,
            Text("cos".to_string()),
            Literal(LiteralValue::Integer(2)),
            RGroup,
            RGroup,
            RGroup
        ],
        tokens
    );

    println!("tokens: {tokens:?}");
}

#[test]
fn test_parser_errors() {
    assert_eq!(Err(LispParseError::EmptyString), Lisp::parse(""));
    assert_eq!(
        Err(LispParseError::IncorrectBrackets),
        Lisp::parse(")(+ 1 2)")
    );
    assert_eq!(Err(LispParseError::NotAFunction), Lisp::parse("(1 2)"));
    assert_eq!(Err(LispParseError::UnclosedBracket), Lisp::parse("(* 4"));
    assert_eq!(
        Err(LispParseError::IncorrectNumArgs),
        Lisp::parse("(- 1 2 3)")
    );
}

#[test]
fn test_parser() {
    let result = Lisp::parse("(* (+ 1 2 (/ a) (sin 0)) (- b))").unwrap();

    assert_eq!(
        Into::<Expression>::into(Variadic::new(
            [
                Variadic::new(
                    [
                        Literal::new(1).into(),
                        Literal::new(2).into(),
                        Unary::new(Variable::new("a").into(), UnaryKind::Reciprocal).into(),
                        Unary::new(Literal::new(0).into(), UnaryKind::named("sin").into()).into()
                    ],
                    VariadicKind::Addition
                )
                .into(),
                Unary::new(Variable::new("b").into(), UnaryKind::Negation).into()
            ],
            VariadicKind::Multiplication
        )),
        result
    );

    println!("result: {result}");
}

#[test]
fn test_shared_identifiers() {
    let exp1 = Lisp::parse("(+ a b)").unwrap();
    let exp2 = Lisp::parse("(+ b c)").unwrap();

    assert_eq!(
        exp1,
        Variadic::new(
            [Variable::new("a").into(), Variable::new("b").into()],
            VariadicKind::Addition
        )
        .into()
    );
    assert_eq!(
        exp2,
        Variadic::new(
            [Variable::new("b").into(), Variable::new("c").into()],
            VariadicKind::Addition
        )
        .into()
    );

    let exp3 = Lisp::parse("(sin 1)").unwrap();
    let exp4 = Lisp::parse("(sin 2)").unwrap();

    assert_eq!(
        exp3,
        Unary::new(Literal::new(1).into(), UnaryKind::named("sin").into()).into()
    );
    assert_eq!(
        exp4,
        Unary::new(Literal::new(2).into(), UnaryKind::named("sin").into()).into()
    )
}

#[test]
fn test_printer() {
    let exp = Variadic::new(
        [
            Literal::new(42).into(),
            Variadic::new(
                [
                    Unary::new(Variable::new("a").into(), UnaryKind::Negation).into(),
                    Unary::new(Variable::new("b").into(), UnaryKind::Reciprocal).into(),
                    Unary::new(Variable::new("c").into(), UnaryKind::named("func1")).into(),
                ],
                VariadicKind::Multiplication,
            )
            .into(),
        ],
        VariadicKind::Addition,
    )
    .into();

    let result = Lisp::format(&exp);

    assert_eq!(result, "(+ (* (- a) (/ b) (func1 c)) 42)".to_string());

    println!("result: {result}");
}

#[test]
fn test_inverse() {
    let expr = Lisp::parse("(* (+ 1 2 (/ a) (sin 0)) (- b))").unwrap();
    assert_eq!(expr, Lisp::parse(&Lisp::format(&expr)).unwrap());
}
