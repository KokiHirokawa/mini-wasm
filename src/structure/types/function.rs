use crate::structure::types::value::ValType;

#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    pub parameters: Vec<ValType>,
    pub results: Vec<ValType>,
}
