use crate::execution::instance::ModuleInst;
use crate::structure::instructions::expression::Instr;
use std::rc::Weak;

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
    Frame(Frame),
}

#[derive(Debug)]
pub struct Label {
    argument_arity: u32,
    instructions: Vec<Instr>,
}

#[derive(Debug)]
pub struct Frame {
    pub return_arity: u32,
    pub locals: Vec<Val>,
    // pub module_inst: Weak<ModuleInst>,
}

#[derive(Debug, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}
