use crate::structure::modules::export::Export;
use crate::structure::modules::function::Func;
use crate::structure::types::function::FuncType;

#[derive(Debug)]
pub struct Module {
    pub types: Vec<FuncType>,
    pub funcs: Vec<Func>,
    pub exports: Vec<Export>,
}