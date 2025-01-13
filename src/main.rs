use std::fs::File;
use std::io::Read;
use crate::decoder::Decoder;
use crate::execution::instance::{alloc_module, invoke, Store, Value};

mod decoder;
mod structure;
mod execution;

fn main() {
    let mut file = File::open("./add.wasm").unwrap();
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
        vec![Value::I32(1), Value::I32(2)],
    );
}
