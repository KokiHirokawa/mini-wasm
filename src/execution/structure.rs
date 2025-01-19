use crate::structure::instructions::expression::Instr;
use crate::structure::modules::function::Func;
use crate::structure::types::function::FuncType;

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

#[derive(Debug)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
}

#[derive(Debug, PartialEq)]
pub struct FuncInst {
    pub type_: FuncType,
    // module: Weak<ModuleInst>,
    pub code: Func,
}

pub type Addr = u32;
pub type FuncAddr = Addr;
