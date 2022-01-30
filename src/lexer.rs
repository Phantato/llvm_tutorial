use std::fs::File;
use std::io::prelude::*;
use std::mem::replace;

use crate::operator::Operator;
use crate::token::*;
use crate::util::*;

pub struct Lexer<'f> {
    source: &'f File,
    buffer: [u8; 1],
}

impl<'f> Lexer<'f> {
    pub fn new(source: &'f mut File) -> Lexer<'f> {
        let mut buffer = [0u8; 1];
        source.read(&mut buffer).unwrap();
        Lexer { source, buffer }
    }

    fn consume_char(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.source.read(&mut buffer).unwrap();
        replace(&mut self.buffer, buffer)[0]
    }

    fn look_ahead(&self) -> &u8 {
        &self.buffer[0]
    }

    fn emit_op(&mut self) -> Token {
        match self.consume_char() {
            40 => Token::LeftParenthesis,
            41 => Token::RightParenthesis,
            42 => Token::Operator(Operator::Mul),
            43 => Token::Operator(Operator::Add),
            44 => Token::Comma,
            45 => Token::Operator(Operator::Sub),
            47 => Token::Operator(Operator::Div),
            ch @ _ => panic!("{} is not valid here", ch),
        }
    }

    pub fn emit_token(&mut self) -> Token {
        while is_space(self.look_ahead()) {
            self.consume_char();
        }
        match self.look_ahead() {
            0 => Token::Eof,
            ch @ _ if is_digit(ch) => {
                let mut number: usize = 0;
                while is_digit(self.look_ahead()) {
                    number *= 10;
                    number += usize::from(self.consume_char() - 48);
                }
                Token::Number(number)
            }
            ch @ _ if is_alpha(ch) => {
                let mut str: Vec<u8> = Vec::new();
                while is_alnum(self.look_ahead()) {
                    str.push(self.consume_char());
                }
                match &str[..] {
                    b"def" => Token::Def,
                    b"extern" => Token::Extern,
                    _ => Token::Identifier(str),
                }
            }
            _ => self.emit_op(),
        }
    }
}
