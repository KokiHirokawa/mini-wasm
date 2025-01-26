use crate::execution::structure::{
    AdministrativeInstruction, Frame, FuncAddr, FuncInst, Runtime, Stack, StackValue, Store, Val,
};
use crate::structure::instructions::expression::Instr;
use crate::structure::modules::export::ExportDesc;
use crate::structure::modules::function::Func;
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::NumType;
use crate::structure::types::value::ValType;
use std::cell::RefCell;
use std::rc::Rc;

pub fn invoke(store: Store, module: &ModuleInst, func_name: String, values: Vec<Val>) {
    let Some(export_inst) = module.exports.iter().find(|e| e.name == func_name) else {
        return;
    };
    let ExternVal::Func(func_address) = &export_inst.value;

    let func_inst = &store.funcs[*func_address as usize];
    let func_type = &func_inst.type_;
    if func_type.parameters.len() != values.len() {
        println!("{:?}", func_type.parameters);
        println!("number of arguments mismatch");
        return;
    }

    let mut stack = Stack::new();

    let dummy_frame = Frame {
        return_arity: 0,
        locals: Vec::new(),
    };
    stack.push(StackValue::Frame(Rc::new(RefCell::new(dummy_frame))));

    for value in values {
        stack.push(StackValue::Value(value));
    }

    let mut runtime = Runtime {
        store: store.clone(),
        stack,
        current_frame: None,
    };
    runtime.run(AdministrativeInstruction::Invoke(*func_address));

    let mut results = Vec::new();
    for _ in 0..func_type.results.len() {
        results.push(runtime.stack.pop());
    }

    // pop the dummy frame
    runtime.stack.pop();

    println!("👻 {:?}", results);
}

pub fn instantiate(store: &mut Store, module: Module) -> ModuleInst {
    let mut module_inst = ModuleInst {
        types: module.types.clone(),
        func_addrs: Vec::new(),
        exports: Vec::new(),
    };

    let mut func_addrs = Vec::new();
    for func in &module.funcs {
        let func_addr = allocate_function(store, func.clone(), &module_inst);
        func_addrs.push(func_addr);
    }

    let mut exports = Vec::new();
    for export in module.exports {
        let export_inst = ExportInst {
            name: export.name,
            value: match export.desc {
                ExportDesc::Func(func_index) => ExternVal::Func(func_index),
            },
        };
        exports.push(export_inst);
    }

    module_inst.func_addrs.extend(func_addrs);
    module_inst.exports.extend(exports);

    module_inst
}

fn allocate_function(store: &mut Store, func: Func, module_inst: &ModuleInst) -> FuncAddr {
    let func_inst = FuncInst {
        type_: module_inst.types[func.type_ as usize].clone(),
        code: func,
    };
    let addr = store.funcs.len() as u32;
    store.funcs.push(func_inst);
    addr
}

#[derive(Debug, PartialEq)]
pub struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub exports: Vec<ExportInst>,
}

#[derive(Debug, PartialEq)]
struct ExportInst {
    name: String,
    value: ExternVal,
}

#[derive(Debug, PartialEq)]
enum ExternVal {
    Func(FuncAddr),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::instructions::expression::{Expr, Instr};
    use crate::structure::modules::export::{Export, ExportDesc};
    use crate::structure::types::value::{NumType, ValType};

    #[test]
    fn test_empty() {
        let store = Store { funcs: Vec::new() };
        let module = Module {
            types: Vec::new(),
            funcs: Vec::new(),
            exports: Vec::new(),
        };
        let (store, module_inst) = alloc_module(store, module);
        assert_eq!(store.funcs, Vec::new());
        assert_eq!(module_inst.func_addrs, Vec::new());
        assert_eq!(module_inst.types, Vec::new());
        assert_eq!(module_inst.exports, Vec::new());
    }

    #[test]
    fn test_func() {
        let store = Store { funcs: Vec::new() };
        let module = Module {
            types: vec![FuncType {
                parameters: vec![ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }],
            funcs: vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
            }],
            exports: vec![Export {
                name: "add".to_string(),
                desc: ExportDesc::Func(0),
            }],
        };
        let (store, module_inst) = alloc_module(store, module);
        assert_eq!(
            store.funcs,
            vec![FuncInst {
                type_: FuncType {
                    parameters: vec![ValType::NumType(NumType::I32)],
                    results: vec![ValType::NumType(NumType::I32)],
                },
                code: Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
                }
            }]
        );
        assert_eq!(
            module_inst.types,
            vec![FuncType {
                parameters: vec![ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }]
        );
        assert_eq!(module_inst.func_addrs, vec![0]);
        assert_eq!(
            module_inst.exports,
            vec![ExportInst {
                name: "add".to_string(),
                value: ExternVal::Func(0)
            },]
        );
    }
}
