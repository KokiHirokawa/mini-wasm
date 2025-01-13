use crate::structure::modules::indice::LocalIdx;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr(pub Vec<Instr>);

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    I32Eqz,
    I32Eq,
    I32Clz,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32Extend8S,
    LocalGet(LocalIdx),
}