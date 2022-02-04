use llvm::builder::Builder;
use llvm::context::Context;
use llvm::module::Module;
use llvm::passes::PassManager;
use llvm::types::{BasicMetadataTypeEnum, IntType};
use llvm::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue};
use llvm::{IntPredicate, OptimizationLevel};

use crate::ast::*;
use crate::operator::*;
use crate::parser::*;
use crate::util::str_from_u8;

use std::collections::HashMap;
use std::str;

pub struct CodeGen<'ctx, 'a> {
    context: &'ctx Context,
    parser: Parser<'a>,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    fpm: &'a PassManager<FunctionValue<'ctx>>,
    symbol_table: HashMap<Vec<u8>, BasicValueEnum<'ctx>>,
    parsed_buffer: Vec<u8>,
}

impl<'ctx, 'a> CodeGen<'ctx, 'a> {
    pub fn new(
        parser: Parser<'a>,
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        fpm: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
    ) -> CodeGen<'ctx, 'a> {
        CodeGen {
            context,
            parser,
            builder,
            module,
            fpm,
            symbol_table: HashMap::new(),
            parsed_buffer: Vec::new(),
        }
    }

    fn run_anon_fn(&mut self, body: Expr) {
        let proto = Prototype::default();
        let anon_module = self.context.create_module("__anon_module");
        self.emit_fn_code(proto, body, &anon_module);

        let ee = anon_module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        ee.add_module(self.module).unwrap();
        let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn() -> usize>("__anon_fn") };
        let compiled_fn = match maybe_fn {
            Ok(f) => f,
            Err(err) => panic!("!> Error during execution: {:?}", err),
        };
        unsafe {
            println!(
                "{} => {}",
                str_from_u8(&self.parsed_buffer),
                compiled_fn.call()
            );
        }
        ee.remove_module(self.module).unwrap();
    }

    pub fn emit_and_run(&mut self) -> Option<()> {
        match self.consume_node() {
            Some(fun) => Some(match (fun.prototype, fun.body) {
                (Some(proto), Some(body)) => {
                    self.emit_fn_code(proto, body, self.module);
                }
                (None, Some(body)) => {
                    self.run_anon_fn(body);
                }
                (Some(proto), None) => {
                    self.emit_proto_type(proto.name, proto.args, self.module);
                }
                (None, None) => {
                    panic!("Unsupposed to see a function without nither prototype nor body!")
                }
            }),
            None => None,
        }
    }

    #[inline]
    fn consume_node(&mut self) -> Option<Function> {
        match self.parser.emit_node() {
            Some((fun, buf)) => {
                self.parsed_buffer = buf;
                Some(fun)
            }
            None => None,
        }
    }

    #[inline]
    fn usize_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
        // .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
    }

    fn emit_op_code(
        &self,
        op: Operator,
        left: Expr,
        right: Expr,
        parent: &FunctionValue,
        module: &Module<'ctx>,
    ) -> IntValue<'ctx> {
        let lhs = self.emit_value_code(left, parent, module);
        let rhs = self.emit_value_code(right, parent, module);
        match op {
            Operator::Add => self.builder.build_int_add(lhs, rhs, "tmpadd"),
            Operator::Sub => self.builder.build_int_sub(lhs, rhs, "tmpsub"),
            Operator::Mul => self.builder.build_int_mul(lhs, rhs, "tmpmul"),
            Operator::Les => self
                .builder
                .build_int_compare(IntPredicate::ULT, lhs, rhs, "tmpcmp"),
            // _ => panic!("Operator not supported "),
        }
    }

    fn emit_call_code(
        &self,
        name: &str,
        args: Vec<Expr>,
        parent: &FunctionValue,
        module: &Module<'ctx>,
    ) -> IntValue<'ctx> {
        match self.module.get_function(name) {
            Some(mut fn_val) => {
                if module != self.module {
                    fn_val = module.add_function(name, fn_val.get_type(), None);
                }
                if fn_val.count_params() as usize != args.len() {
                    panic!("Incorrect # of arguments passed");
                }

                let mut compiled_args = Vec::with_capacity(args.len());
                for arg in args {
                    compiled_args.push(self.emit_value_code(arg, parent, module));
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

    fn emit_condition_code(
        &self,
        predicate: Expr,
        consequence: Expr,
        alternative: Expr,
        parent: &FunctionValue,
        module: &Module<'ctx>,
    ) -> IntValue<'ctx> {
        // entry
        let cond = self.emit_value_code(predicate, parent, module);
        let zero_const = self.context.bool_type().const_int(0, false);

        //blocks
        let cond = self
            .builder
            .build_int_compare(IntPredicate::NE, cond, zero_const, "cond");
        let then = self.context.append_basic_block(*parent, "then");
        let other = self.context.append_basic_block(*parent, "other");
        let merge = self.context.append_basic_block(*parent, "merge");
        self.builder.build_conditional_branch(cond, then, other);

        // then
        self.builder.position_at_end(then);
        let then_val = self.emit_value_code(consequence, parent, module);
        self.builder.build_unconditional_branch(merge);

        let then = self.builder.get_insert_block().unwrap();

        // build else block
        self.builder.position_at_end(other);
        let other_val = self.emit_value_code(alternative, parent, module);
        self.builder.build_unconditional_branch(merge);

        let other = self.builder.get_insert_block().unwrap();

        // emit merge block
        self.builder.position_at_end(merge);

        let phi = self.builder.build_phi(self.usize_type(), "iftmp");

        phi.add_incoming(&[(&then_val, then), (&other_val, other)]);

        phi.as_basic_value().into_int_value()
    }

    fn emit_value_code(
        &self,
        expr: Expr,
        parent: &FunctionValue,
        module: &Module<'ctx>,
    ) -> IntValue<'ctx> {
        match expr {
            Expr::Number(value) => self
                .usize_type()
                .const_int(value.try_into().unwrap(), false),
            Expr::Variable(name) => match self.symbol_table.get(&name) {
                Some(var) => var.into_int_value(),
                // self
                //     .builder
                //     .build_load(
                //         (*var).into_pointer_value(),
                //         str_from_u8(name),
                //     )
                //     .into_int_value(),
                None => panic!(
                    "Could not find variable `{}` in symbol table",
                    str_from_u8(&name)
                ),
            },
            Expr::Binary { op, lhs, rhs } => self.emit_op_code(op, *lhs, *rhs, parent, module),
            Expr::Call { name, args } => {
                self.emit_call_code(str_from_u8(&name), args, parent, module)
            }
            Expr::Condition {
                predicate,
                then,
                other,
            } => self.emit_condition_code(*predicate, *then, *other, parent, module),
        }
    }

    fn emit_proto_type(
        &self,
        name: Vec<u8>,
        args: Vec<Vec<u8>>,
        module: &Module<'ctx>,
    ) -> FunctionValue<'ctx> {
        let ret_type = self.usize_type();
        let args_types = std::iter::repeat(ret_type)
            .take(args.len())
            .map(|ty| ty.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.usize_type().fn_type(args_types, false);
        let fn_val = module.add_function(str_from_u8(&name), fn_type, None);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_int_value().set_name(str_from_u8(&args[i]));
        }
        fn_val
    }

    fn emit_fn_code(
        &mut self,
        prototype: Prototype,
        body: Expr,
        module: &Module<'ctx>,
    ) -> FunctionValue<'ctx> {
        let name = prototype.name;
        let args = prototype.args;
        let args_num = args.len();
        let fn_val = match module.get_function(str_from_u8(&name)) {
            Some(fn_val) => fn_val,
            None => self.emit_proto_type(name, args, module),
        };
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);
        self.symbol_table.reserve(args_num);

        for arg in fn_val.get_param_iter() {
            // let arg_name = str_from_u8(args[i]);
            // let alloca = self.create_entry_block_alloca(arg_name);
            // self.builder.build_store(alloca, arg);
            self.symbol_table.insert(
                arg.into_int_value().get_name().to_bytes().to_vec(),
                arg.into(),
            );
        }

        let body = self.emit_value_code(body, &fn_val, module);
        self.builder.build_return(Some(&body));
        if fn_val.verify(true) {
            self.fpm.run_on(&fn_val);
            fn_val
        } else {
            unsafe {
                fn_val.delete();
            }
            panic!("Invalid generated function.")
        }
    }
}
