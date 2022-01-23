use std::io::prelude::*;
use std::fs::File;
use std::mem::replace;

use crate::token::*;
use crate::operator::Operator;
use crate::util::*;

pub struct Lexer {
    source: File,
    buffer: [u8; 1]
}

impl Lexer {

    pub fn new(mut source: File) -> Lexer {
        let mut buffer = [0u8; 1];
        source.read(&mut buffer).unwrap();
        Lexer {source, buffer}
    }

    fn get_char(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.source.read(&mut buffer).unwrap();
        replace(&mut self.buffer, buffer)[0]
    }

    fn look_ahead(&self) -> &u8 {
        &self.buffer[0]
    }

    fn get_op(&mut self) -> Token {
        match self.get_char() {
            40 => Token::LeftParenthesis,
            41 => Token::RightParenthesis,
            42 => Token::Operator( Operator::Mul ),
            43 => Token::Operator( Operator::Add ),
            44 => Token::Comma,
            45 => Token::Operator( Operator::Sub ),
            47 => Token::Operator( Operator::Div ),
            ch @ _ => panic!("{} is not valid here", ch)
        }
    }

    pub fn get_token(&mut self) -> Token {
        while is_space(self.look_ahead()) {
            self.get_char();
        }
        let tok = match self.look_ahead() {
            0 => Token::Eof,
            ch @ _ if is_digit(ch) => {
                let mut number:usize = 0;
                while is_digit(self.look_ahead()) {
                    number *= 10;
                    number += usize::from(self.get_char() - 48);
                }
                Token::Number(number)
            },
            ch @ _ if is_alpha(ch) => {
                let mut str: Vec<u8> = Vec::new();
                while is_alnum(self.look_ahead()) {
                    str.push(self.get_char());
                }
                match str[..] {
                    // for `def`
                    [100, 101, 102] => Token::Def,
                    // for `extern`
                    [101, 120, 116, 101, 114, 110] => Token::Extern,
                    _ => Token::Identifier(Box::from(str))
                }
            },
            _ => self.get_op()
        };
        // println!("{:?}", tok);
        tok
    }
}