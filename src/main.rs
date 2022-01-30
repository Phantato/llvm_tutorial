extern crate inkwell as llvm;

mod ast;
mod codeGenor;
mod lexer;
mod operator;
mod parser;
mod token;
mod util;

use std::fs;

use ast::Node;
use lexer::Lexer;

use parser::Parser;

fn main() {
    let mut parser = Parser::new(Lexer::new(fs::File::open("Input").unwrap()));
    println!("Start");
    loop {
        let node = parser.emit_node();
        match node {
            Node::Eof => break,
            _ => println!("{:?}", node),
        }
    }
}
