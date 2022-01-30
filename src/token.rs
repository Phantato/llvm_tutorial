use crate::operator::Operator;

#[derive(Debug)]
pub enum Token {
    Operator(Operator),
    Eof,
    Def,
    Extern,
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Identifier(Vec<u8>),
    Number(usize),
}
