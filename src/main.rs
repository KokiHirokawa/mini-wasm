use crate::decoder::Decoder;
use crate::execution::instance::{instantiate, invoke};
use crate::execution::structure::{Store, Val};
use clap::Parser;
use std::fs::File;
use std::io::Read;

mod decoder;
mod execution;
mod structure;

#[derive(Parser)]
#[command(version)]
struct Cli {
    filename: String,

    /// Run exported function by name
    #[arg(short = 'r', long = "run-export", value_name = "FUNCTION")]
    exported_function: String, // FIXME: Option<String>

    /// Add argument to an exported function execution
    #[arg(short = 'a', long = "argument", value_name = "ARGUMENT")]
    arguments: Vec<i32>, // FIXME: Vec<String>
}

fn main() {
    let cli = Cli::parse();

    let mut file = File::open(cli.filename).unwrap();
    let mut input = Vec::new();
    file.read_to_end(&mut input).unwrap();

    let mut decoder = Decoder::new(&input);
    let module = decoder.decode().unwrap();

    let mut store = Store { funcs: Vec::new() };
    let module_inst = instantiate(&mut store, module);

    let arguments = cli.arguments.iter().map(|x| Val::I64(*x as i64)).collect();
    invoke(&store, &module_inst, cli.exported_function, arguments);
}
