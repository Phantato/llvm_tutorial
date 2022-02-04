use std::mem::replace;

use crate::operator::Operator;
use crate::token::*;
use crate::util::*;

pub struct Lexer<'a> {
    source: &'a Vec<u8>,
    index: usize,
    parsed_buffer: Vec<u8>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.emit_token())
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &Vec<u8>) -> Lexer {
        Lexer {
            source,
            index: 0,
            parsed_buffer: Vec::new(),
        }
    }

    fn consume_char(&mut self) -> &u8 {
        if self.index < self.source.len() {
            if !is_space(&self.source[self.index]) {
                self.parsed_buffer.push(self.source[self.index]);
            }
            self.index += 1;
            &self.source[self.index - 1]
        } else {
            &0
        }
    }

    fn look_ahead(&self) -> &u8 {
        if self.index < self.source.len() {
            &self.source[self.index]
        } else {
            &0
        }
    }

    fn emit_op(&mut self) -> Token {
        match self.consume_char() {
            40 => Token::LeftParenthesis,
            41 => Token::RightParenthesis,
            42 => Token::Operator(Operator::Mul),
            43 => Token::Operator(Operator::Add),
            44 => Token::Comma,
            45 => Token::Operator(Operator::Sub),
            // 47 => Token::Operator(Operator::Div),
            60 => Token::Operator(Operator::Les),
            ch @ _ => panic!("{} is not valid here", ch),
        }
    }

    pub fn pop_parsed_buffer(&mut self) -> Vec<u8> {
        replace(&mut self.parsed_buffer, Vec::new())
    }

    pub fn emit_token(&mut self) -> (Token, Vec<u8>) {
        while is_space(self.look_ahead()) {
            self.consume_char();
        }
        let tok = match self.look_ahead() {
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
                    str.push(*self.consume_char());
                }
                match &str[..] {
                    b"def" => Token::Def,
                    b"extern" => Token::Extern,
                    b"if" => Token::If,
                    b"then" => Token::Then,
                    b"else" => Token::Else,
                    _ => Token::Identifier(str),
                }
            }
            _ => self.emit_op(),
        };
        (tok, self.pop_parsed_buffer())
    }
}
