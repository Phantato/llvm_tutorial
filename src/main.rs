mod ast;
mod token;
mod lexer;
mod parser;
mod util;
mod operator;

use std::fs;

use ast::Node;
use lexer::Lexer;
use parser::Parser;

fn main() {

    // let mut sql = OpenOptions::new().append(true).open("sql.txt").unwrap();
    // let mut player = 43599;
    // // let skills = [1, 6, 7, 8, 9, 11, 13, 14, 17, 21, 22, 28, 29, 30, 34, 35, 36, 38, 39, 40, 41, 45, 47, 53, 55, 56, 57, 58, 60, 61, 63, 68, 70, 71, 72];
    // let skills = [1, 6, 7, 8, 9, 11, 13, 14, 17, 21, 22, 28, 29, 30, 34, 35, 36, 38, 39, 40, 41, 45, 47, 53, 55, 56, 57, 58, 60, 61, 63, 68, 70, 71, 72];

    // while player < 43609 {
    //     for skill in skills.iter() {
    //         writeln!(&mut sql, "INSERT INTO `main`.`bb_player_skills` (`idPlayerListing`, `idSkillListing`) VALUES ('{}', '{}');", player, skill).unwrap();
    //     }
    //     player += 1;
    // }
    // return;
    
    let mut parser = Parser::new(Lexer::new(fs::File::open("Input").unwrap()));
    println!("Start");
    loop {
        let node = parser.get_node();
        match node {
            Node::Eof => break,
            _ => println!("{:?}", node),
        }
    }
}
