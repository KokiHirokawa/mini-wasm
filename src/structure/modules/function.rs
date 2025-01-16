use crate::structure::instructions::expression::Expr;
use crate::structure::modules::indice::TypeIdx;
use crate::structure::types::value::ValType;

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<ValType>,
    pub body: Expr,
}
