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
    Identifier(Box<[u8]>),
    Number(usize),
}

pub fn is_binary_op(token: &Token) -> bool {
    match token {
        Token::Operator(op) => match op {
            Operator::Assign
            | Operator::Add
            | Operator::Sub
            | Operator::Mul
            | Operator::Div
            | Operator::Les => true,
            _ => false,
        },
        _ => false,
    }
}
