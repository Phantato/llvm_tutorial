use crate::operator::Operator;
#[derive(Debug)]
pub enum Node {
    Number(usize),
    Variable(Vec<u8>),
    Binary {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Call {
        name: Vec<u8>,
        args: Vec<Node>,
    },
    Prototype {
        name: Vec<u8>,
        args: Vec<Vec<u8>>,
    },
    Function {
        prototype: Box<Node>,
        body: Box<Node>,
    },
    Eof,
}
