use std::io::Read;
use crate::execution::instance::{alloc_module, invoke, Store, Value};
use crate::structure::instructions::expression::{Expr, Instr};
use crate::structure::modules::export::{Export, ExportDesc};
use crate::structure::modules::function::Func;
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::{NumType, ValType};

mod decoder;
mod structure;
mod execution;

fn main() {
    let store = Store { funcs: Vec::new() };
    let module = Module {
        types: vec![
            FuncType {
                parameters: vec![ValType::NumType(NumType::I32), ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }
        ],
        funcs: vec![
            Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
            }
        ],
        exports: vec![Export { name: "add".to_string(), desc: ExportDesc::Func(0) }],
    };
    let (store, module_inst) = alloc_module(store, module);
    invoke(
        &store,
        &module_inst,
        "add".to_string(),
        vec![Value::I32(1), Value::I32(2)],
    );
}
