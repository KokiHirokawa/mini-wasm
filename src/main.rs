use crate::decoder::Decoder;
use crate::execution::instance::{alloc_module, invoke, Store};
use crate::execution::structure::Val;
use std::fs::File;
use std::io::Read;

mod decoder;
mod execution;
mod structure;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let function_name = std::env::args().nth(2).expect("no function name given");

    let lhs = std::env::args().nth(3).expect("no lhs given").parse::<i32>().unwrap();
    let rhs = std::env::args().nth(4).expect("no rhs given").parse::<i32>().unwrap();

    let path = format!("./tests/inputs/{}", filename);
    let mut file = File::open(path).unwrap();
    let mut input = Vec::new();
    file.read_to_end(&mut input).unwrap();

    let mut decoder = Decoder::new(&input);
    let module = decoder.decode().unwrap();

    let store = Store { funcs: Vec::new() };
    let (store, module_inst) = alloc_module(store, module);
    invoke(
        &store,
        &module_inst,
        function_name,
        vec![Val::I32(lhs), Val::I32(rhs)],
    );
}
