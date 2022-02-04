use std::mem::replace;

use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::*;

pub struct Parser<'lexer> {
    lexer: Lexer<'lexer>,
    buffer: Token,
    parsed_buffer: Vec<Vec<u8>>,
    ahead_buffer: Vec<u8>,
}

impl<'lexer> Parser<'lexer> {
    pub fn new(mut lexer: Lexer) -> Parser {
        let (buffer, ahead_buffer) = lexer.emit_token();
        Parser {
            lexer,
            buffer,
            parsed_buffer: Vec::new(),
            ahead_buffer,
        }
    }

    fn consume_token(&mut self) -> Token {
        let (tok, source) = self.lexer.emit_token();
        self.parsed_buffer
            .push(replace(&mut self.ahead_buffer, source));
        replace(&mut self.buffer, tok)
    }

    fn look_ahead(&self) -> &Token {
        &self.buffer
    }

    fn parse_primary(&mut self) -> Expr {
        match self.consume_token() {
            Token::Number(value) => Expr::Number(value),
            Token::LeftParenthesis => {
                let ret = self.parse_expr();
                match self.consume_token() {
                    Token::RightParenthesis => ret,
                    _ => panic!("Expected to see an right parenthesis!"),
                }
            }
            Token::Identifier(name) => match self.look_ahead() {
                Token::LeftParenthesis => {
                    self.consume_token();
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_expr());
                        match self.consume_token() {
                            Token::Comma => continue,
                            Token::RightParenthesis => break,
                            _ => panic!("Expected to see `)` or `,` in arguments list"),
                        };
                    }
                    Expr::Call { name, args }
                }
                _ => Expr::Variable(name),
            },
            Token::If => {
                let predicate = Box::new(self.parse_expr());
                if let Token::Then = self.look_ahead() {
                    self.consume_token();
                    let then = Box::new(self.parse_expr());
                    if let Token::Else = self.look_ahead() {
                        self.consume_token();
                        let other = Box::new(self.parse_expr());
                        Expr::Condition {
                            predicate,
                            then,
                            other,
                        }
                    } else {
                        panic!("Expected to see else here");
                    }
                } else {
                    panic!("Expected to see then here.")
                }
            }
            tok @ _ => panic!("Expected to see a primary type here, but got {:?}", tok),
        }
    }

    fn parse_binary_expr(&mut self, mut lhs: Expr, precedence: i8) -> Expr {
        while let Token::Operator(op) = self.look_ahead() {
            if precedence > op.precedence() {
                break;
            }
            if let Token::Operator(op) = self.consume_token() {
                let mut rhs = self.parse_primary();
                while let Token::Operator(ahead) = self.look_ahead() {
                    if ahead.is_binary_op() && op < *ahead {
                        let next_prec = op.precedence() + (*ahead > op) as i8;
                        rhs = self.parse_binary_expr(rhs, next_prec);
                    } else {
                        break;
                    }
                }
                lhs = Expr::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
        }
        lhs
    }

    fn parse_expr(&mut self) -> Expr {
        let lhs = self.parse_primary();
        self.parse_binary_expr(lhs, 0)
    }

    fn parse_prototypes(&mut self) -> Prototype {
        if let Token::Identifier(name) = self.consume_token() {
            if let Token::LeftParenthesis = self.consume_token() {
                let mut args = Vec::new();
                loop {
                    match self.consume_token() {
                        Token::Comma => continue,
                        Token::Identifier(name) => args.push(name),
                        Token::RightParenthesis => break,
                        tok @ _ => panic!("Unexpected token here {:?}", tok),
                    }
                }
                Prototype { name, args }
            } else {
                panic!("Expected to see `(` in `prototype");
            }
        } else {
            panic!("Expected to see an identifier here");
        }
    }

    fn parse_def(&mut self) -> Function {
        self.consume_token();
        let prototype = Some(self.parse_prototypes());
        let body = Some(self.parse_expr());
        Function { prototype, body }
    }

    fn parse_extern(&mut self) -> Function {
        self.consume_token();
        let prototype = Some(self.parse_prototypes());
        Function {
            prototype,
            body: None,
        }
    }

    fn parse_top_level_expr(&mut self) -> Function {
        Function {
            prototype: None,
            body: Some(self.parse_expr()),
        }
    }

    pub fn pop_parsed_buffer(&mut self) -> Vec<u8> {
        replace(&mut self.parsed_buffer, Vec::new())
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn emit_node(&mut self) -> Option<(Function, Vec<u8>)> {
        let ret = match self.look_ahead() {
            Token::Eof => None,
            Token::Def => Some(self.parse_def()),
            Token::Extern => Some(self.parse_extern()),
            _ => Some(self.parse_top_level_expr()),
        };
        match ret {
            Some(fun) => Some((fun, self.pop_parsed_buffer())),
            None => None,
        }
    }
}
