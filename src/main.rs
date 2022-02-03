extern crate inkwell as llvm;

use llvm::context::Context;
use llvm::passes::PassManager;
// use llvm::execution_engine::{ExecutionEngine, JitFunction};
use llvm::module::Module;
use llvm::values::FunctionValue;

mod ast;
mod code_generator;
mod lexer;
mod operator;
mod parser;
mod token;
mod util;

use std::fs;
use std::io::Read;

use code_generator::CodeGen;
use lexer::Lexer;
use parser::Parser;

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
    // preload modules
    for path in std::env::args().skip(1) {
        let mut source = fs::File::open(&path).unwrap();
        let context = Context::create();
        let module = context.create_module(&path);
        let builder = context.create_builder();
        let fpm = fn_optimizer(&module);

        let mut buf = Vec::new();
        source.read_to_end(&mut buf).unwrap();

        let lexer = Lexer::new(buf);
        let parser = Parser::new(lexer);
        let mut code_generator = CodeGen::new(parser, &context, &builder, &fpm, &module);
        while let Some(_) = code_generator.emit_and_run() {}
    }
}
