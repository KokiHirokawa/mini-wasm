#[derive(Debug, Clone, PartialEq)]
pub enum ValType {
    NumType(NumType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}
