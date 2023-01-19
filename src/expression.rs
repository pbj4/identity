mod multiset;

use multiset::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Expression {
    Concrete(Concrete),
    Variable(Variable),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum Concrete {
    Variadic(Variadic),
    Unary(Unary),
    Literal(Literal),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Variadic {
    pub terms: MultiSet<Expression>,
    pub kind: VariadicKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub enum VariadicKind {
    Addition,
    Multiplication,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Unary {
    pub argument: Box<Expression>,
    pub kind: UnaryKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum UnaryKind {
    Negation,
    Reciprocal,
    Named { id: FuncId },
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct FuncId(String);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Literal {
    pub value: LiteralValue,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum LiteralValue {
    Integer(i32),
    Constant(String),
}

// ids MUST have no repeats across statements
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Variable {
    pub id: VarId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct VarId(String);

impl From<Variadic> for Expression {
    fn from(value: Variadic) -> Self {
        Self::Concrete(Concrete::Variadic(value))
    }
}

impl From<Unary> for Expression {
    fn from(value: Unary) -> Self {
        Self::Concrete(Concrete::Unary(value))
    }
}

impl From<Literal> for Expression {
    fn from(value: Literal) -> Self {
        Self::Concrete(Concrete::Literal(value))
    }
}

impl From<Variable> for Expression {
    fn from(value: Variable) -> Self {
        Self::Variable(value)
    }
}

impl Variadic {
    pub fn new(terms: impl IntoIterator<Item = Expression>, kind: VariadicKind) -> Self {
        Self {
            terms: terms.into_iter().collect(),
            kind,
        }
    }
}

impl Unary {
    pub fn new(argument: Expression, kind: UnaryKind) -> Self {
        Self {
            argument: Box::new(argument),
            kind,
        }
    }
}

impl UnaryKind {
    pub fn named(id: impl ToString) -> Self {
        let id = FuncId::new(id);
        Self::Named { id }
    }
}

impl FuncId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<String> for FuncId {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Literal {
    pub fn new(value: impl Into<LiteralValue>) -> Self {
        let value = value.into();
        Self { value }
    }
}

impl From<i32> for LiteralValue {
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl From<&str> for LiteralValue {
    fn from(value: &str) -> Self {
        Self::Constant(value.to_string())
    }
}

impl Variable {
    pub fn new(id: impl ToString) -> Self {
        let id = VarId::new(id);
        Self { id }
    }
}

impl VarId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<String> for VarId {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
