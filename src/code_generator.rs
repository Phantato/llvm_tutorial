use llvm::builder::Builder;
use llvm::context::Context;
use llvm::execution_engine::{ExecutionEngine, JitFunction};
use llvm::module::Module;
use llvm::types::{BasicMetadataTypeEnum, IntType};
use llvm::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use llvm::{IntPredicate, OptimizationLevel};

use crate::ast::*;
use crate::operator::*;
use crate::parser::*;

use std::collections::HashMap;
use std::str;

pub struct CodeGen<'ctx, 'a> {
    source: &'ctx mut Parser<'ctx>,
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    // fpm: PassManager<FunctionValue<'ctx>>,
    execution_engine: ExecutionEngine<'ctx>,
    symbol_table: HashMap<Vec<u8>, BasicValueEnum<'ctx>>,
}

impl<'ctx, 'a> CodeGen<'ctx, 'a> {
    pub fn new(
        source: &'ctx mut Parser<'ctx>,
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
    ) -> CodeGen<'ctx, 'a> {
        CodeGen {
            source,
            context,
            builder,
            module,
            // fpm: PassManager::create(module),
            execution_engine: module
                .create_jit_execution_engine(OptimizationLevel::None)
                .unwrap(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn emit_code(&mut self) -> Option<FunctionValue<'ctx>> {
        match self.consume_node() {
            Node::Function { prototype, body } => Some(self.emit_fn_code(*prototype, *body)),
            Node::Eof => None,
            node @ _ => panic!("nothing to generate for {:?}", node),
        }
    }

    #[inline]
    fn consume_node(&mut self) -> Node {
        self.source.emit_node()
    }

    #[inline]
    fn usize_type(&self) -> IntType<'ctx> {
        self.context
            .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
    }

    fn emit_op_code(&self, op: Operator, left: Node, right: Node) -> IntValue<'ctx> {
        let lhs = self.emit_value_code(left);
        let rhs = self.emit_value_code(right);
        match op {
            Operator::Add => self.builder.build_int_add(lhs, rhs, "tmpadd"),
            Operator::Sub => self.builder.build_int_sub(lhs, rhs, "tmpsub"),
            Operator::Mul => self.builder.build_int_mul(lhs, rhs, "tmpmul"),
            Operator::Les => self
                .builder
                .build_int_compare(IntPredicate::ULT, lhs, rhs, "tmpcmp"),
            _ => panic!("Operator not supported "),
        }
    }

    fn emit_call_code(&self, name: &str, args: Vec<Node>) -> IntValue<'ctx> {
        match self.module.get_function(name) {
            Some(fn_val) => {
                if fn_val.count_params() as usize != args.len() {
                    panic!("Incorrect # arguments passed");
                }

                let mut compiled_args = Vec::with_capacity(args.len());
                for arg in args {
                    compiled_args.push(self.emit_value_code(arg));
                }

                let argsv: Vec<BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .by_ref()
                    .map(|&val| val.into())
                    .collect();

                match self
                    .builder
                    .build_call(fn_val, argsv.as_slice(), "tmp")
                    .try_as_basic_value()
                    .left()
                {
                    Some(value) => value.into_int_value(),
                    None => panic!("Invalid call produced."),
                }
            }
            None => panic!("Could not find function `{}`", name),
        }
    }

    fn emit_value_code(&self, node: Node) -> IntValue<'ctx> {
        match node {
            Node::Number(value) => self
                .usize_type()
                .const_int(value.try_into().unwrap(), false),
            Node::Variable(name) => match self.symbol_table.get(&name) {
                Some(var) => var.into_int_value(),
                // self
                //     .builder
                //     .build_load(
                //         (*var).into_pointer_value(),
                //         String::from_utf8(name).unwrap().as_str(),
                //     )
                //     .into_int_value(),
                None => panic!(
                    "Could not find variable `{}` in symbol table",
                    String::from_utf8(name).unwrap().as_str()
                ),
            },
            Node::Binary { op, lhs, rhs } => self.emit_op_code(op, *lhs, *rhs),
            Node::Call { name, args } => {
                self.emit_call_code(std::str::from_utf8(&name).unwrap(), args)
            }
            _ => panic!("Expected to see a value here"),
        }
    }

    fn emit_proto_type(&self, name: Vec<u8>, args: Vec<Vec<u8>>) -> FunctionValue<'ctx> {
        let ret_type = self.usize_type();
        let args_types = std::iter::repeat(ret_type)
            .take(args.len())
            .map(|ty| ty.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.usize_type().fn_type(args_types, false);
        let fn_val = self
            .module
            .add_function(&String::from_utf8(name).unwrap(), fn_type, None);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_int_value()
                .set_name(str::from_utf8(&args[i]).unwrap());
        }
        fn_val
    }

    fn emit_fn_code(&mut self, prototype: Node, body: Node) -> FunctionValue<'ctx> {
        if let Node::Prototype { name, args } = prototype {
            let args_num = args.len();
            if let None = self.module.get_function(str::from_utf8(&name).unwrap()) {
                self.emit_proto_type(name.clone(), args);
            }
            if let Some(fn_val) = self.module.get_function(str::from_utf8(&name).unwrap()) {
                let entry = self.context.append_basic_block(fn_val, "entry");
                self.builder.position_at_end(entry);
                self.symbol_table.reserve(args_num);

                for arg in fn_val.get_param_iter() {
                    // let arg_name = str::from_utf8(args[i]).unwrap();
                    // let alloca = self.create_entry_block_alloca(arg_name);
                    // self.builder.build_store(alloca, arg);
                    self.symbol_table.insert(
                        arg.into_int_value().get_name().to_bytes().to_vec(),
                        arg.into(),
                    );
                }
                let body = self.emit_value_code(body);
                self.builder.build_return(Some(&body));
                if fn_val.verify(true) {
                    fn_val
                } else {
                    unsafe {
                        fn_val.delete();
                    }
                    panic!("Invalid generated function.")
                }
            } else {
                panic!("Prototype not found")
            }
        } else {
            panic!("Supposed to see a prototype")
        }
    }
}
