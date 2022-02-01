use std::mem::replace;

use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::*;

pub struct Parser<'lex> {
    source: &'lex mut Lexer<'lex>,
    buffer: Token,
}

impl<'lex> Parser<'lex> {
    pub fn new(source: &'lex mut Lexer<'lex>) -> Parser<'lex> {
        let buffer = source.emit_token();
        Parser { source, buffer }
    }

    fn consume_token(&mut self) -> Token {
        replace(&mut self.buffer, self.source.emit_token())
    }

    fn look_ahead(&self) -> &Token {
        &self.buffer
    }

    fn parse_primary(&mut self) -> Node {
        match self.consume_token() {
            Token::Number(value) => Node::Number(value),
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
                    Node::Call { name, args }
                }
                _ => Node::Variable(name),
            },
            Token::If => {
                let predicate = Box::new(self.parse_expr());
                if let Token::Then = self.look_ahead() {
                    self.consume_token();
                    let then = Box::new(self.parse_expr());
                    if let Token::Else = self.look_ahead() {
                        self.consume_token();
                        let other = Box::new(self.parse_expr());
                        Node::Condition {
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

    fn parse_binary_expr(&mut self, mut lhs: Node, precedence: i8) -> Node {
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
                lhs = Node::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
        }
        lhs
    }

    fn parse_expr(&mut self) -> Node {
        let lhs = self.parse_primary();
        self.parse_binary_expr(lhs, 0)
    }

    fn parse_prototypes(&mut self) -> Node {
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
                Node::Prototype { name, args }
            } else {
                panic!("Expected to see `(` in `prototype");
            }
        } else {
            panic!("Expected to see an identifier here");
        }
    }

    fn parse_def(&mut self) -> Node {
        self.consume_token();
        let prototype = Box::new(self.parse_prototypes());
        let body = Box::new(self.parse_expr());
        Node::Function { prototype, body }
    }

    fn parse_extern(&mut self) -> Node {
        self.consume_token();
        self.parse_prototypes()
    }

    fn parse_top_level_expr(&mut self) -> Node {
        Node::Function {
            prototype: Box::new(Node::Prototype {
                name: Vec::from("__anon_expr"),
                args: Vec::new(),
            }),
            body: Box::new(self.parse_expr()),
        }
    }

    pub fn emit_node(&mut self) -> Node {
        match self.look_ahead() {
            Token::Eof => Node::Eof,
            Token::Def => self.parse_def(),
            Token::Extern => self.parse_extern(),
            _ => self.parse_top_level_expr(),
        }
    }
}
