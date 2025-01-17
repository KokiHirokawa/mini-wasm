use crate::decoder::Decoder;
use crate::execution::instance::{alloc_module, invoke, Store};
use crate::execution::structure::Val;
use std::fs::File;
use std::io::Read;

mod decoder;
mod execution;
mod structure;

fn main() {
    let mut file = File::open("./tests/inputs/i32.add.wasm").unwrap();
    let mut input = Vec::new();
    file.read_to_end(&mut input).unwrap();

    println!("input: {:?}", input);

    let mut decoder = Decoder::new(&input);
    let module = decoder.decode().unwrap();

    dbg!(&module);

    let store = Store { funcs: Vec::new() };
    let (store, module_inst) = alloc_module(store, module);
    invoke(
        &store,
        &module_inst,
        "add".to_string(),
        vec![Val::I32(1), Val::I32(2)],
    );
}
