use crate::operator::Operator;

#[derive(Debug)]
pub enum Expr {
    Number(usize),
    Variable(Vec<u8>),
    Binary {
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: Vec<u8>,
        args: Vec<Expr>,
    },
    Condition {
        predicate: Box<Expr>,
        then: Box<Expr>,
        other: Box<Expr>,
    },
}
#[derive(Debug)]
pub struct Function {
    pub prototype: Option<Prototype>,
    pub body: Option<Expr>,
}

#[derive(Debug)]
pub struct Prototype {
    pub name: Vec<u8>,
    pub args: Vec<Vec<u8>>,
}

impl Default for Prototype {
    fn default() -> Self {
        Prototype {
            name: Vec::from("__anon_fn"),
            args: Vec::new(),
        }
    }
}
