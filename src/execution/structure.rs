use crate::structure::instructions::expression::Instr;
use crate::structure::modules::function::Func;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::{NumType, ValType};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Runtime {
    pub store: Store,
    pub stack: Stack,
    pub current_frame: Option<Rc<RefCell<Frame>>>,
}

impl Runtime {
    pub fn run(&mut self, administrative_instruction: AdministrativeInstruction) {
        match administrative_instruction {
            AdministrativeInstruction::Invoke(func_addr) => {
                self.invoke_function(func_addr);
            }
            AdministrativeInstruction::Label(label) => {
                self.execute_instructions(label);
            }
            AdministrativeInstruction::Frame => {}
        }
    }

    fn invoke_function(&mut self, func_addr: FuncAddr) {
        let func_inst = self.store.funcs[func_addr as usize].clone();
        let func_type = func_inst.type_;

        let mut locals = Vec::new();
        for _ in 0..func_type.parameters.len() {
            let value = match self.stack.pop() {
                Some(StackValue::Value(value)) => value,
                _ => panic!("parameters are not on the top of the stack"),
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

        let return_arity = func_type.results.len() as u32;
        let frame = Rc::new(RefCell::new(Frame {
            return_arity,
            locals,
        }));
        // self.stack.push(StackValue::Frame(Rc::clone(&frame)));
        self.current_frame = Some(Rc::clone(&frame));

        let label = Label {
            argument_arity: return_arity,
            instructions: func_inst.code.body.0.clone(),
        };
        // push label to the stack
        // self.stack.push(StackValue::Label(label));

        self.run(AdministrativeInstruction::Label(label));
    }

    fn execute_instructions(&mut self, label: Label) {
        for instr in label.instructions {
            self.execute_instruction(instr);
        }
    }

    fn execute_instruction(&mut self, instr: Instr) {
        match instr {
            Instr::If(block_type, instructions1, instructions2) => {
                let x = match self.stack.pop() {
                    Some(StackValue::Value(Val::I32(x))) => x as i64,
                    Some(StackValue::Value(Val::I64(x))) => x,
                    _ => panic!(),
                };
                let label = Label {
                    argument_arity: 0,
                    instructions: if x != 0 { instructions1 } else { instructions2 },
                };
                // push label to the stack
                // self.stack.push(StackValue::Label(label));
                self.execute_instructions(label);
            }
            Instr::Call(idx) => {
                self.invoke_function(idx);
            }
            Instr::LocalGet(idx) => {
                if let Some(ref current_frame) = self.current_frame {
                    let value = current_frame.borrow().locals[idx as usize].clone();
                    self.stack.push(StackValue::Value(value));
                } else {
                    panic!()
                }
            }
            Instr::I32Const(x) => {
                self.stack.push(StackValue::Value(Val::I32(x)));
            }
            Instr::I64Const(x) => {
                self.stack.push(StackValue::Value(Val::I64(x)));
            }
            Instr::I32Add => {
                self.execute_i32_binop(|lhs, rhs| lhs + rhs);
            }
            Instr::I32Sub => {
                self.execute_i32_binop(|lhs, rhs| lhs - rhs);
            }
            Instr::I32Mul => {
                self.execute_i32_binop(|lhs, rhs| lhs * rhs);
            }
            Instr::I32DivS => {
                self.execute_i32_binop(|lhs, rhs| lhs / rhs);
            }
            Instr::I32DivU => {
                self.execute_i32_binop(|lhs, rhs| lhs / rhs);
            }
            Instr::I32RemS => {
                self.execute_i32_binop(|lhs, rhs| lhs % rhs);
            }
            Instr::I32RemU => {
                self.execute_i32_binop(|lhs, rhs| lhs % rhs);
            }
            Instr::I32And => {
                self.execute_i32_binop(|lhs, rhs| lhs & rhs);
            }
            Instr::I32Or => {
                self.execute_i32_binop(|lhs, rhs| lhs | rhs);
            }
            Instr::I32Xor => {
                self.execute_i32_binop(|lhs, rhs| lhs ^ rhs);
            }
            Instr::I32Shl => {
                self.execute_i32_binop(|lhs, rhs| lhs << rhs);
            }
            Instr::I32ShrS => {
                self.execute_i32_binop(|lhs, rhs| lhs >> rhs);
            }
            Instr::I32ShrU => {
                self.execute_i32_binop(|lhs, rhs| lhs >> rhs);
            }
            Instr::I32Rotl => {
                self.execute_i32_binop(|lhs, rhs| (lhs << rhs) | (rhs >> (32 - rhs)));
            }
            Instr::I32Rotr => {
                self.execute_i32_binop(|lhs, rhs| (lhs >> rhs) | lhs << (32 - rhs));
            }
            Instr::I32Clz => {
                self.execute_i32_unop(|x| x.leading_zeros() as i32);
            }
            Instr::I32Ctz => {
                self.execute_i32_unop(|x| x.trailing_zeros() as i32);
            }
            Instr::I32Popcnt => {
                self.execute_i32_unop(|x| x.count_ones() as i32);
            }
            Instr::I32Extend8S => {
                self.execute_i32_unop(|x| (x as i8) as i32);
            }
            Instr::I32Extend16S => {
                self.execute_i32_unop(|x| (x as i16) as i32);
            }
            Instr::I32Eqz => {
                self.execute_i32_unop(|x| x.eq(&0) as i32);
            }
            Instr::I32Eq => {
                self.execute_i32_binop(|lhs, rhs| lhs.eq(&rhs) as i32);
            }
            Instr::I32Ne => {
                self.execute_i32_binop(|lhs, rhs| lhs.ne(&rhs) as i32);
            }
            Instr::I32LtS => {
                self.execute_i32_binop(|lhs, rhs| lhs.lt(&rhs) as i32);
            }
            Instr::I32LtU => self.execute_i32_binop(|lhs, rhs| {
                let lhs = lhs as u32;
                let rhs = rhs as u32;
                lhs.lt(&rhs) as i32
            }),
            Instr::I32LeS => {
                self.execute_i32_binop(|lhs, rhs| (lhs <= rhs) as i32);
            }
            Instr::I32LeU => {
                self.execute_i32_binop(|lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.le(&rhs) as i32
                });
            }
            Instr::I32GtS => {
                self.execute_i32_binop(|lhs, rhs| lhs.gt(&rhs) as i32);
            }
            Instr::I32GtU => {
                self.execute_i32_binop(|lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.gt(&rhs) as i32
                });
            }
            Instr::I32GeS => {
                self.execute_i32_binop(|lhs, rhs| lhs.ge(&lhs) as i32);
            }
            Instr::I32GeU => {
                self.execute_i32_binop(|lhs, rhs| {
                    let lhs = lhs as u32;
                    let rhs = rhs as u32;
                    lhs.ge(&rhs) as i32
                });
            }
            Instr::I64Eqz => {
                self.execute_i64_unop(|x| x.eq(&0) as i64);
            }
            Instr::I64Sub => {
                self.execute_i64_binop(|lhs, rhs| lhs - rhs);
            }
            Instr::I64Mul => {
                self.execute_i64_binop(|lhs, rhs| lhs * rhs);
            }
            _ => unimplemented!("{:?}", instr),
        }
    }

    fn execute_i32_unop<F>(&mut self, f: F)
    where
        F: FnOnce(i32) -> i32,
    {
        let x = match self.stack.pop() {
            Some(StackValue::Value(Val::I32(value))) => value,
            _ => panic!(),
        };
        let result = f(x);
        self.stack.push(StackValue::Value(Val::I32(result)));
    }

    fn execute_i64_unop<F>(&mut self, f: F)
    where
        F: FnOnce(i64) -> i64,
    {
        let x = match self.stack.pop() {
            Some(StackValue::Value(Val::I64(value))) => value,
            _ => panic!(),
        };
        let result = f(x);
        self.stack.push(StackValue::Value(Val::I64(result)));
    }

    fn execute_i32_binop<F>(&mut self, f: F)
    where
        F: FnOnce(i32, i32) -> i32,
    {
        let rhs = match self.stack.pop() {
            Some(StackValue::Value(Val::I32(value))) => value,
            _ => panic!(),
        };
        let lhs = match self.stack.pop() {
            Some(StackValue::Value(Val::I32(value))) => value,
            _ => panic!(),
        };
        let result = f(lhs, rhs);
        self.stack.push(StackValue::Value(Val::I32(result)));
    }

    fn execute_i64_binop<F>(&mut self, f: F)
    where
        F: FnOnce(i64, i64) -> i64,
    {
        let rhs = match self.stack.pop() {
            Some(StackValue::Value(Val::I64(value))) => value,
            _ => panic!(),
        };
        let lhs = match self.stack.pop() {
            Some(StackValue::Value(Val::I64(value))) => value,
            _ => panic!(),
        };
        let result = f(lhs, rhs);
        self.stack.push(StackValue::Value(Val::I64(result)));
    }
}

#[derive(Debug)]
pub struct Stack {
    values: Vec<StackValue>,
}

impl Stack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, val: StackValue) {
        self.values.push(val)
    }

    pub fn pop(&mut self) -> Option<StackValue> {
        self.values.pop()
    }
}

#[derive(Debug)]
pub enum StackValue {
    Value(Val),
    Label(Label),
    Frame(Rc<RefCell<Frame>>),
}

#[derive(Debug)]
pub struct Label {
    pub argument_arity: u32,
    pub instructions: Vec<Instr>,
}

#[derive(Debug)]
pub struct Frame {
    pub return_arity: u32,
    pub locals: Vec<Val>,
    // pub module_inst: Weak<ModuleInst>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, Clone)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncInst {
    pub type_: FuncType,
    // module: Weak<ModuleInst>,
    pub code: Func,
}

pub type Addr = u32;
pub type FuncAddr = Addr;

pub enum AdministrativeInstruction {
    Invoke(FuncAddr),
    Label(Label),
    Frame,
}
