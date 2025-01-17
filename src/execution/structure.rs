use crate::execution::instance::ModuleInst;
use crate::structure::instructions::expression::Instr;
use std::rc::Weak;

#[derive(Debug)]
struct Stack {
    values: Vec<StackValue>,
}

impl Stack {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn push(&mut self, val: StackValue) {
        self.values.push(val)
    }

    fn pop(&mut self) -> Option<StackValue> {
        self.values.pop()
    }
}

#[derive(Debug)]
enum StackValue {
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
    return_arity: u32,
    locals: Vec<Val>,
    module_inst: Weak<ModuleInst>,
}

#[derive(Debug, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}
