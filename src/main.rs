use std::fs::File;
use std::io::Read;
use crate::decoder::Decoder;

mod decoder;

fn main() {
    let mut file = File::open("add.wasm").expect("failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("failed to read file");

    let mut decoder = Decoder::new(buffer);
    _ = decoder.decode();
}
