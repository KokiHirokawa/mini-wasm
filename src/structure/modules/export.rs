use crate::structure::modules::indice::FuncIdx;

pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

pub enum ExportDesc {
    Func(FuncIdx),
}