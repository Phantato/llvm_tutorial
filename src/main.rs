extern crate inkwell as llvm;

use llvm::context::Context;
use llvm::module::Module;
use llvm::passes::PassManager;
use llvm::values::FunctionValue;

mod ast;
mod code_generator;
mod executor;
mod lexer;
mod operator;
mod parser;
mod token;
mod util;

use std::fs;
use std::io::{stdin, stdout, Read, Write};

use ast::Function;
use code_generator::CodeGen;
use lexer::Lexer;
use parser::Parser;
use util::*;

macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );

        stdout().flush().expect("Could not flush to standard output.");
    };
}

fn fn_optimizer<'m, 'ctx>(module: &'m Module<'ctx>) -> PassManager<FunctionValue<'ctx>> {
    let fpm = PassManager::create(module);

    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    fpm.add_promote_memory_to_register_pass();
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();

    fpm.initialize();

    fpm
}

fn main() {
    let context = Context::create();
    let module = context.create_module("preload");
    let builder = context.create_builder();
    let fpm = fn_optimizer(&module);
    // preload modules
    for path in std::env::args().skip(1) {
        let mut source = fs::File::open(&path).unwrap();

        let mut buf = Vec::new();
        source.read_to_end(&mut buf).unwrap();

        let mut lex = Lexer::new(&buf);
        let mut par = Parser::new(&mut lex);
        let mut code_generator = CodeGen::new(&mut par, &context, &module, &builder, &fpm);
        while let Some(_) = code_generator.emit_and_run() {}
    }
}
