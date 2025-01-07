use std::fs::File;
use std::io;
use std::io::Read;

fn main() -> io::Result<()> {
    let mut f = File::open("./empty.wasm")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    println!("{:?}", buffer);

    Ok(())
}
