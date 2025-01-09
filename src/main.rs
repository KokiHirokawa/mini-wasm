use std::fs::File;
use std::io;
use std::io::Read;

fn main() -> io::Result<()> {
    let mut f = File::open("./empty.wasm")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let magic_number = &buffer[0..4];
    if magic_number != b"\0asm" {
        panic!("invalid wasm file")
    }
    println!("magic: {:?}", magic_number);

    let version = &buffer[4..8];
    println!("version: {:?}", version);

    Ok(())
}
