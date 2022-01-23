use crate::operator::Operator;
#[derive(Debug)]
pub enum Node {
    Number(usize),
    Variable(Box<[u8]>),
    Binary(BinaryNode),
    Call(CallNode),
    Prototype(PrototypeNode),
    Function(FunctionNode),
    Eof
}

#[derive(Debug)]
pub struct BinaryNode {pub op: Operator, pub lhs: Box<Node>, pub rhs: Box<Node>}
#[derive(Debug)]
pub struct CallNode {pub name: Box<[u8]>, pub args: Vec<Node>}
#[derive(Debug)]
pub struct PrototypeNode {pub name: Box<[u8]>, pub args: Vec<Box<[u8]>>}
#[derive(Debug)]
pub struct FunctionNode {pub prototype: PrototypeNode, pub body: Box<Node>}
