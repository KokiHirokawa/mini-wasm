use crate::structure::modules::indice::LocalIdx;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr(pub Vec<Instr>);

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    I32Add,
    LocalGet(LocalIdx),
}