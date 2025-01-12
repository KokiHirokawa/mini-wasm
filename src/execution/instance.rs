use std::iter::zip;
use crate::structure::instructions::expression::Instr;
use crate::structure::modules::export::ExportDesc;
use crate::structure::modules::function::Func;
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::ValType;

pub fn invoke(
    store: &Store,
    module: &ModuleInst,
    func_name: String,
    values: Vec<Value>,
) {
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

    for instr in &func_inst.code.body.0 {
        match instr {
            Instr::I32Add => {
                let lhs = match stack.pop() {
                    Some(StackValue::Value(Value::I32(x))) => x,
                    None => return,
                };
                let rhs = match stack.pop() {
                    Some(StackValue::Value(Value::I32(x))) => x,
                    None => return,
                };
                stack.push(StackValue::Value(Value::I32(lhs + rhs)));
            }
            Instr::LocalGet(idx) => {
                let val = &values[0];
                stack.push(StackValue::Value(val.clone()));
            }
        }
        println!("{:?}", stack);
    }
}

pub fn alloc_module(
    store: Store,
    module: Module,
) -> (Store, ModuleInst) {
    let mut store = store;

    let mut types = Vec::new();
    let mut func_addrs = Vec::new();
    for (i, (func_type, func)) in zip(&module.types, &module.funcs).enumerate() {
        let func_inst = FuncInst {
            type_: func_type.clone(),
            code: func.clone(),
        };
        store.funcs.push(func_inst);

        types.push(func_type.clone());
        func_addrs.push(i as FuncAddr);
    }

    let mut exports = Vec::new();
    for export in module.exports {
        let export_inst = ExportInst {
            name: export.name,
            value: match export.desc {
                ExportDesc::Func(func_index) => { ExternVal::Func(func_index) }
            },
        };
        exports.push(export_inst);
    }

    let module_inst = ModuleInst {
        types,
        func_addrs,
        exports,
    };
    (store, module_inst)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I32(i32),
}

#[derive(Debug)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
}

#[derive(Debug, PartialEq)]
pub struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub exports: Vec<ExportInst>,
}

#[derive(Debug, PartialEq)]
pub struct FuncInst {
    type_: FuncType,
    // module: Weak<ModuleInst>,
    code: Func,
}

type Addr = u32;
type FuncAddr = Addr;

#[derive(Debug, PartialEq)]
struct ExportInst {
    name: String,
    value: ExternVal,
}

#[derive(Debug, PartialEq)]
enum ExternVal {
    Func(FuncAddr),
}

#[derive(Debug, PartialEq)]
struct Stack {
    values: Vec<StackValue>,
}

impl Stack {
    fn new() -> Self { Self { values: Vec::new() } }

    fn push(&mut self, val: StackValue) { self.values.push(val) }

    fn pop(&mut self) -> Option<StackValue> { self.values.pop() }
}

#[derive(Debug, PartialEq)]
enum StackValue {
    Value(Value),
}

#[cfg(test)]
mod tests {
    use crate::structure::instructions::expression::{Expr, Instr};
    use crate::structure::modules::export::{Export, ExportDesc};
    use crate::structure::types::value::{NumType, ValType};
    use super::*;

    #[test]
    fn test_empty() {
        let store = Store { funcs: Vec::new() };
        let module = Module { types: Vec::new(), funcs: Vec::new(), exports: Vec::new() };
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
            types: vec![
                FuncType {
                    parameters: vec![ValType::NumType(NumType::I32)],
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
        assert_eq!(
            store.funcs,
            vec![
                FuncInst {
                    type_: FuncType {
                        parameters: vec![ValType::NumType(NumType::I32)],
                        results: vec![ValType::NumType(NumType::I32)],
                    },
                    code: Func {
                        type_: 0,
                        locals: Vec::new(),
                        body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
                    }
                }
            ]
        );
        assert_eq!(
            module_inst.types,
            vec![
                FuncType {
                    parameters: vec![ValType::NumType(NumType::I32)],
                    results: vec![ValType::NumType(NumType::I32)],
                }
            ]
        );
        assert_eq!(module_inst.func_addrs, vec![0]);
        assert_eq!(
            module_inst.exports,
            vec![
                ExportInst { name: "add".to_string(), value: ExternVal::Func(0) },
            ]
        );
    }
}