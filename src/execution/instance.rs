use crate::execution::structure::{
    Frame, FuncAddr, FuncInst, Label, Stack, StackValue, Store, Val,
};
use crate::structure::instructions::expression::{BlockType, Instr};
use crate::structure::modules::export::ExportDesc;
use crate::structure::modules::function::Func;
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::NumType;
use crate::structure::types::value::ValType;
use std::iter::zip;

pub fn invoke(store: &Store, module: &ModuleInst, func_name: String, values: Vec<Val>) {
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
    stack.push(StackValue::Frame(dummy_frame));

    for value in values {
        stack.push(StackValue::Value(value));
    }

    // invoke the function
    {
        let return_arity = func_type.results.len() as u32;

        let argument_arity = func_type.parameters.len();
        let mut locals = Vec::new();
        for _ in 0..argument_arity {
            let value = match stack.pop() {
                Some(StackValue::Value(value)) => value,
                _ => panic!(),
            };
            locals.push(value);
            locals.reverse();

            for local in &func_inst.code.locals {
                let local = match local {
                    ValType::NumType(NumType::I32) => Val::I32(0),
                    ValType::NumType(NumType::I64) => Val::I64(0),
                    ValType::NumType(NumType::F32) => Val::F32(0.0),
                    ValType::NumType(NumType::F64) => Val::F64(0.0),
                };
                locals.push(local);
            }
        }

        let frame = Frame {
            return_arity,
            locals,
        };
        // push frame to the stack
        // stack.push(StackValue::Frame(frame));

        let label = Label {
            argument_arity: return_arity,
            instructions: func_inst.code.body.0.clone(),
        };
        // push label to the stack
        // stack.push(StackValue::Label(label));

        // jump to the start of the instruction sequence
        run(&mut stack, &frame, &label, &store);
    }

    let mut results = Vec::new();
    for _ in 0..func_type.results.len() {
        results.push(stack.pop());
    }

    // pop the dummy frame
    stack.pop();

    println!("👻 {:?}", results);
}

pub fn run(stack: &mut Stack, frame: &Frame, label: &Label, store: &Store) {
    for instr in label.instructions.clone() {
        match instr {
            Instr::If(block_type, instructions1, instructions2) => {
                let x = match stack.pop() {
                    Some(StackValue::Value(Val::I32(x))) => x as i64,
                    Some(StackValue::Value(Val::I64(x))) => x,
                    _ => panic!(),
                };
                let label = Label {
                    argument_arity: 0,
                    instructions: if x != 0 { instructions1 } else { instructions2 },
                };
                run(stack, frame, &label, &store);
            }
            Instr::Call(idx) => {
                let func_inst = &store.funcs[idx as usize];
                let func_type = &func_inst.type_;
                let return_arity = func_type.results.len() as u32;

                let argument_arity = func_type.parameters.len();
                let mut locals = Vec::new();
                for _ in 0..argument_arity {
                    let value = match stack.pop() {
                        Some(StackValue::Value(value)) => value,
                        _ => panic!(),
                    };
                    locals.push(value);
                    locals.reverse();

                    for local in &func_inst.code.locals {
                        let local = match local {
                            ValType::NumType(NumType::I32) => Val::I32(0),
                            ValType::NumType(NumType::I64) => Val::I64(0),
                            ValType::NumType(NumType::F32) => Val::F32(0.0),
                            ValType::NumType(NumType::F64) => Val::F64(0.0),
                        };
                        locals.push(local);
                    }
                }

                let frame = Frame {
                    return_arity,
                    locals,
                };
                // push frame to the stack
                // stack.push(StackValue::Frame(frame));

                let label = Label {
                    argument_arity: return_arity,
                    instructions: func_inst.code.body.0.clone(),
                };

                run(stack, &frame, &label, &store);
            }
            Instr::LocalGet(idx) => {
                let value = frame.locals[idx as usize].clone();
                stack.push(StackValue::Value(value));
            }
            Instr::I32Const(x) => {
                stack.push(StackValue::Value(Val::I32(x)));
            }
            Instr::I64Const(x) => {
                stack.push(StackValue::Value(Val::I64(x)));
            }
            Instr::I32Add => {
                run_binop(stack, |lhs, rhs| lhs + rhs);
            }
            Instr::I32Sub => {
                run_binop(stack, |lhs, rhs| lhs - rhs);
            }
            Instr::I32Mul => {
                run_binop(stack, |lhs, rhs| lhs * rhs);
            }
            Instr::I32DivS => {
                run_binop(stack, |lhs, rhs| lhs / rhs);
            }
            Instr::I32DivU => {
                run_binop(stack, |lhs, rhs| lhs / rhs);
            }
            Instr::I32RemS => {
                run_binop(stack, |lhs, rhs| lhs % rhs);
            }
            Instr::I32RemU => {
                run_binop(stack, |lhs, rhs| lhs % rhs);
            }
            Instr::I32And => {
                run_binop(stack, |lhs, rhs| lhs & rhs);
            }
            Instr::I32Or => {
                run_binop(stack, |lhs, rhs| lhs | rhs);
            }
            Instr::I32Xor => {
                run_binop(stack, |lhs, rhs| lhs ^ rhs);
            }
            Instr::I32Shl => {
                run_binop(stack, |lhs, rhs| lhs << rhs);
            }
            Instr::I32ShrS => {
                run_binop(stack, |lhs, rhs| lhs >> rhs);
            }
            Instr::I32ShrU => {
                run_binop(stack, |lhs, rhs| lhs >> rhs);
            }
            Instr::I32Rotl => {
                run_binop(stack, |lhs, rhs| (lhs << rhs) | (rhs >> (32 - rhs)));
            }
            Instr::I32Rotr => {
                run_binop(stack, |lhs, rhs| (lhs >> rhs) | lhs << (32 - rhs));
            }
            Instr::I32Clz => {
                run_unop(stack, |x| x.leading_zeros() as i32);
            }
            Instr::I32Ctz => {
                run_unop(stack, |x| x.trailing_zeros() as i32);
            }
            Instr::I32Popcnt => {
                run_unop(stack, |x| x.count_ones() as i32);
            }
            Instr::I32Extend8S => {
                run_unop(stack, |x| (x as i8) as i32);
            }
            Instr::I32Extend16S => {
                run_unop(stack, |x| (x as i16) as i32);
            }
            Instr::I32Eqz => {
                run_unop(stack, |x| x.eq(&0) as i32);
            }
            Instr::I32Eq => {
                run_binop(stack, |lhs, rhs| lhs.eq(&rhs) as i32);
            }
            Instr::I32Ne => {
                run_binop(stack, |lhs, rhs| lhs.ne(&rhs) as i32);
            }
            Instr::I32LtS => {
                run_binop(stack, |lhs, rhs| lhs.lt(&rhs) as i32);
            }
            Instr::I32LtU => run_binop(stack, |lhs, rhs| {
                let lhs = lhs as u32;
                let rhs = rhs as u32;
                lhs.lt(&rhs) as i32
            }),
            Instr::I32LeS => {
                run_binop(stack, |lhs, rhs| (lhs <= rhs) as i32);
            }
            Instr::I32LeU => {
                run_binop(stack, |lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.le(&rhs) as i32
                });
            }
            Instr::I32GtS => {
                run_binop(stack, |lhs, rhs| lhs.gt(&rhs) as i32);
            }
            Instr::I32GtU => {
                run_binop(stack, |lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.gt(&rhs) as i32
                });
            }
            Instr::I32GeS => {
                run_binop(stack, |lhs, rhs| lhs.ge(&lhs) as i32);
            }
            Instr::I32GeU => {
                run_binop(stack, |lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.ge(&rhs) as i32
                });
            }
            Instr::I64Eqz => {
                run_unop_i64(stack, |x| x.eq(&0) as i64);
            }
            Instr::I64Sub => {
                run_binop_i64(stack, |lhs, rhs| lhs - rhs);
            }
            Instr::I64Mul => {
                run_binop_i64(stack, |lhs, rhs| lhs * rhs);
            }
            _ => unimplemented!("{:?}", instr),
        }
    }
}

pub fn run_binop<F>(stack: &mut Stack, f: F)
where
    F: FnOnce(i32, i32) -> i32,
{
    let rhs = match stack.pop() {
        Some(StackValue::Value(Val::I32(value))) => value,
        _ => panic!(),
    };
    let lhs = match stack.pop() {
        Some(StackValue::Value(Val::I32(value))) => value,
        _ => panic!(),
    };
    let result = f(lhs, rhs);
    stack.push(StackValue::Value(Val::I32(result)));
}

pub fn run_binop_i64<F>(stack: &mut Stack, f: F)
where
    F: FnOnce(i64, i64) -> i64,
{
    let rhs = match stack.pop() {
        Some(StackValue::Value(Val::I64(value))) => value,
        _ => panic!(),
    };
    let lhs = match stack.pop() {
        Some(StackValue::Value(Val::I64(value))) => value,
        _ => panic!(),
    };
    let result = f(lhs, rhs);
    stack.push(StackValue::Value(Val::I64(result)));
}

pub fn run_unop<F>(stack: &mut Stack, f: F)
where
    F: FnOnce(i32) -> i32,
{
    let x = match stack.pop() {
        Some(StackValue::Value(Val::I32(value))) => value,
        _ => panic!(),
    };
    let result = f(x);
    stack.push(StackValue::Value(Val::I32(result)));
}

pub fn run_unop_i64<F>(stack: &mut Stack, f: F)
where
    F: FnOnce(i64) -> i64,
{
    let x = match stack.pop() {
        Some(StackValue::Value(Val::I64(value))) => value,
        _ => panic!(),
    };
    let result = f(x);
    stack.push(StackValue::Value(Val::I64(result)));
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
