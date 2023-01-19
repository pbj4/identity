use crate::{rewrite::*, text::*};

pub struct Lisp;

// lisp grammar:
// expr := (func expr*) | lit | var
// ruleset := (=> expr expr)*

const COMMENT: &str = ";";

#[derive(Debug, PartialEq)]
pub enum LispParseError {
    EmptyString,
    IncorrectBrackets,
    EmptyFuncBody,
    UnclosedBracket,
    IncorrectNumArgs,
    ReservedOperator,
    NotAFunction,
    ExpectedBracket,
    ExpectedRuleOp,
}

impl ExprTextFormat<LispParseError> for Lisp {
    fn parse_tokens(tokens: &mut VecDeque<SourceToken>) -> Result<Expression, LispParseError> {
        if let Some(token) = tokens.pop_front() {
            match token {
                SourceToken::Literal(l) => Ok(Literal::new(l).into()),
                SourceToken::Text(t) => Ok(Variable::new(t).into()),
                SourceToken::LGroup => {
                    if let Some(func) = tokens.pop_front() {
                        let mut args = Vec::new();

                        loop {
                            match Self::parse_tokens(tokens) {
                                Ok(expr) => args.push(expr),
                                Err(LispParseError::IncorrectBrackets) => break,
                                Err(LispParseError::EmptyString) => {
                                    return Err(LispParseError::UnclosedBracket)
                                }
                                Err(e) => return Err(e),
                            }
                        }

                        if let SourceToken::VariadicOperator(kind) = func {
                            Ok(Variadic::new(args, kind).into())
                        } else if let Ok([arg]) = <_ as TryInto<[_; 1]>>::try_into(args) {
                            if let SourceToken::UnaryOperator(kind) = func {
                                Ok(Unary::new(arg, kind).into())
                            } else if let SourceToken::Text(kind) = func {
                                Ok(Unary::new(arg, UnaryKind::named(kind)).into())
                            } else {
                                Err(LispParseError::NotAFunction)
                            }
                        } else {
                            Err(LispParseError::IncorrectNumArgs)
                        }
                    } else {
                        Err(LispParseError::EmptyFuncBody)
                    }
                }
                SourceToken::RGroup => Err(LispParseError::IncorrectBrackets),
                SourceToken::VariadicOperator(_) => Err(LispParseError::ReservedOperator),
                SourceToken::UnaryOperator(_) => Err(LispParseError::ReservedOperator),
                SourceToken::RuleOperator(_) => {
                    panic!("rules are not allowed to appear in expressions")
                }
            }
        } else {
            Err(LispParseError::EmptyString)
        }
    }

    fn format_expr(expr: &Expression) -> VecDeque<SourceToken> {
        let mut output = VecDeque::new();

        match expr {
            Expression::Concrete(c) => match c {
                Concrete::Variadic(Variadic { terms, kind }) => {
                    output.extend([SourceToken::LGroup, SourceToken::VariadicOperator(*kind)]);

                    for term in terms.iter() {
                        output.push_back(SourceToken::Text(" ".to_string()));
                        output.append(&mut Self::format_expr(term));
                    }

                    output.push_back(SourceToken::RGroup);
                }
                Concrete::Unary(Unary { argument, kind }) => {
                    output.extend([
                        SourceToken::LGroup,
                        SourceToken::UnaryOperator(kind.clone()),
                        SourceToken::Text(" ".to_string()),
                    ]);

                    output.append(&mut Self::format_expr(argument));
                    output.push_back(SourceToken::RGroup);
                }
                Concrete::Literal(Literal { value }) => {
                    output.push_back(SourceToken::Literal(value.clone()));
                }
            },
            Expression::Variable(Variable { id }) => {
                output.push_back(SourceToken::Text(id.as_ref().to_string()))
            }
        }

        output
    }
}

impl RuleTextFormat<LispParseError> for Lisp {
    fn parse_tokens_rule(tokens: &mut VecDeque<SourceToken>) -> Result<Rule, LispParseError> {
        if SourceToken::LGroup != tokens.pop_front().ok_or(LispParseError::EmptyString)? {
            return Err(LispParseError::ExpectedBracket);
        }

        let SourceToken::RuleOperator(kind) = tokens.pop_front().ok_or(LispParseError::EmptyFuncBody)? else {return Err(LispParseError::ExpectedRuleOp)};

        let pattern = Self::parse_tokens(tokens)?;
        let replacement = Self::parse_tokens(tokens)?;

        if SourceToken::RGroup != tokens.pop_front().ok_or(LispParseError::UnclosedBracket)? {
            return Err(LispParseError::IncorrectNumArgs);
        }

        Ok(Rule {
            pattern,
            replacement,
            kind,
        })
    }

    fn format_rule_tokens(rule: &Rule) -> VecDeque<SourceToken> {
        let Rule {
            pattern,
            replacement,
            kind,
        } = rule;
        let mut output = VecDeque::new();

        output.extend([
            SourceToken::LGroup,
            SourceToken::RuleOperator(*kind),
            SourceToken::Text(" ".to_string()),
        ]);

        output.append(&mut Self::format_expr(pattern));

        output.push_back(SourceToken::Text(" ".to_string()));

        output.append(&mut Self::format_expr(replacement));

        output.push_back(SourceToken::RGroup);

        output
    }

    fn comment() -> &'static str {
        COMMENT
    }
}
