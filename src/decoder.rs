use crate::structure::instructions::expression::{Expr, Instr};
use crate::structure::modules::export::{Export, ExportDesc};
use crate::structure::modules::function::Func;
use crate::structure::modules::indice::{FuncIdx, LocalIdx, TypeIdx};
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::ValType;
use crate::structure::types::value::NumType;

#[derive(Debug)]
pub struct DecodingError {}

impl DecodingError {
    fn new() -> Self { Self {} }
}

pub struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Decoder {
        Self { input, pos: 0 }
    }
}

impl Decoder<'_> {
    pub fn decode(&mut self) -> Result<Module, DecodingError> {
        let mut module = Module {
            types: Vec::new(),
            funcs: Vec::new(),
            exports: Vec::new(),
        };

        let magic_number = self.decode_magic_number()?;
        if magic_number != vec![0x00, 0x61, 0x73, 0x6d] {
            return Err(DecodingError::new());
        }

        let version = self.decode_version()?;
        if version != vec![0x01, 0x00, 0x00, 0x00] {
            return Err(DecodingError::new());
        }

        let mut type_idxs = Vec::new();
        while self.pos < self.input.len() {
            let section_id = self.input[self.pos];
            self.pos += 1;

            match section_id {
                1 => {
                    module.types = self.decode_type_section()?;
                    dbg!(&module.types);
                }
                3 => {
                    type_idxs = self.decode_function_section()?;
                    dbg!(&type_idxs);
                }
                7 => {
                    module.exports = self.decode_export_section()?;
                    dbg!(&module.exports);
                }
                10 => {
                    module.funcs = self.decode_code_section(&type_idxs)?;
                    dbg!(&module.funcs);
                }
                _ => {
                    let section_size = self.input[self.pos];
                    self.pos += 1;

                    println!("id: {}, size: {}", section_id, section_size);

                    self.pos += section_size as usize;
                }
            }
        }

        Ok(module)
    }

    fn decode_magic_number(&mut self) -> Result<Vec<u8>, DecodingError> {
        assert_eq!(self.pos, 0);

        let mut magic_number = Vec::new();
        for _ in 0..4 {
            if self.input.len() <= self.pos {
                return Err(DecodingError::new());
            }

            magic_number.push(self.input[self.pos]);
            self.pos += 1;
        }
        Ok(magic_number)
    }

    fn decode_version(&mut self) -> Result<Vec<u8>, DecodingError> {
        assert_eq!(self.pos, 4);

        let mut version = Vec::new();
        for _ in 0..4 {
            if self.input.len() <= self.pos {
                return Err(DecodingError::new());
            }

            version.push(self.input[self.pos]);
            self.pos += 1;
        }
        Ok(version)
    }

    fn decode_type_section(&mut self) -> Result<Vec<FuncType>, DecodingError> {
        let mut types = Vec::new();

        let section_size = self.input[self.pos];
        self.pos += 1;

        println!("size of type section: {}", section_size);

        let num_of_func_types = self.input[self.pos];
        self.pos += 1;

        for i in 0..num_of_func_types {
            if self.input[self.pos] != 0x60 {
                return Err(DecodingError::new());
            }
            self.pos += 1;

            let num_of_parameters = self.input[self.pos];
            self.pos += 1;
            let mut parameters = Vec::new();
            for _ in 0..num_of_parameters {
                let val_type = match self.input[self.pos] {
                    0x7f => ValType::NumType(NumType::I32),
                    0x7e => ValType::NumType(NumType::I64),
                    0x7d => ValType::NumType(NumType::F32),
                    0x7c => ValType::NumType(NumType::F64),
                    _ => unimplemented!("unimplemented value type"),
                };
                parameters.push(val_type);
                self.pos += 1;
            }

            let mut num_of_results = self.input[self.pos];
            self.pos += 1;
            let mut results = Vec::new();
            for _ in 0..num_of_results {
                let val_type = match self.input[self.pos] {
                    0x7f => ValType::NumType(NumType::I32),
                    0x7e => ValType::NumType(NumType::I64),
                    0x7d => ValType::NumType(NumType::F32),
                    0x7c => ValType::NumType(NumType::F64),
                    _ => unimplemented!("unimplemented value type"),
                };
                results.push(val_type);
                self.pos += 1;
            }

            let func_type = FuncType {
                parameters,
                results,
            };
            types.push(func_type);
        }

        Ok(types)
    }

    fn decode_function_section(&mut self) -> Result<Vec<TypeIdx>, DecodingError> {
        let mut idxs = Vec::new();

        let section_size = self.input[self.pos];
        self.pos += 1;

        println!("size of function section: {}", section_size);

        let num_of_idxs = self.input[self.pos];
        self.pos += 1;

        for _ in 0..num_of_idxs {
            // TODO: LEB128 https://webassembly.github.io/spec/core/binary/values.html#integers
            idxs.push(self.input[self.pos] as TypeIdx);
            self.pos += 1;
        }

        Ok(idxs)
    }

    fn decode_export_section(&mut self) -> Result<Vec<Export>, DecodingError> {
        let mut exports = Vec::new();

        let section_size = self.input[self.pos];
        self.pos += 1;

        println!("size of export section: {}", section_size);

        let num_of_exports = self.input[self.pos];
        self.pos += 1;

        for _ in 0..num_of_exports {
            let name_length = self.input[self.pos];
            self.pos += 1;

            let mut name = String::new();
            for _ in 0..name_length {
                name.push(self.input[self.pos] as char);
                self.pos += 1;
            }

            let desc_type = self.input[self.pos];
            self.pos += 1;

            let idx = self.input[self.pos];
            self.pos += 1;

            let desc = match desc_type {
                0x00 => ExportDesc::Func(idx as FuncIdx),
                _ => unimplemented!("unimplemented export desc"),
            };

            let export = Export { name, desc };
            exports.push(export);
        }

        Ok(exports)
    }

    fn decode_code_section(
        &mut self,
        type_idxs: &[TypeIdx],
    ) -> Result<Vec<Func>, DecodingError> {
        let section_size = self.input[self.pos];
        self.pos += 1;

        println!("size of code section: {}", section_size);

        let num_of_funcs = self.input[self.pos];
        self.pos += 1;

        let mut funcs = Vec::new();
        for i in 0..num_of_funcs {
            let size = self.input[self.pos];
            self.pos += 1;

            let num_of_locals = self.input[self.pos];
            self.pos += 1;

            let mut locals = Vec::new();
            for _ in 0..num_of_locals {
                unimplemented!("decode vec(locals)")
            }

            let mut body = Expr(Vec::new());
            loop {
                let byte = self.input[self.pos];
                self.pos += 1;

                if byte == 0x0b {
                    break;
                }

                let instr = match byte {
                    0x20 => {
                        let idx = self.input[self.pos];
                        self.pos += 1;
                        Instr::LocalGet(idx as LocalIdx)
                    },
                    0x6a => Instr::I32Add,
                    _ => unimplemented!("unimplemented instr"),
                };
                body.0.push(instr);
            }

            let func = Func {
                type_: type_idxs[i as usize],
                locals,
                body
            };
            funcs.push(func);
        }

        Ok(funcs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let buffer = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00].to_vec();
        let mut decoder = Decoder::new(buffer);

        let mut module = Module::new();
        module.magic = [0x00, 0x61, 0x73, 0x6d];
        module.version = [0x01, 0x00, 0x00, 0x00];

        assert_eq!(decoder.decode(), Ok(module));
    }

    #[test]
    fn test_add() {
        let mut buffer = [
            0x00, 0x61, 0x73, 0x6d,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x07, 0x01, 0x60,
            0x02, 0x7f, 0x7f, 0x01,
            0x7f, 0x03, 0x02, 0x01,
            0x00, 0x07, 0x07, 0x01,
            0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09,
            0x01, 0x07, 0x00, 0x20,
            0x00, 0x20, 0x01, 0x6a,
            0x0b,
        ].to_vec();
        let mut decoder = Decoder::new(buffer);

        let mut module = Module::new();
        module.magic = [0x00, 0x61, 0x73, 0x6d];
        module.version = [0x01, 0x00, 0x00, 0x00];
        module.type_section = Some(TypeSection {
            function_types: vec![FunctionType {
                params: ResultType {
                    value_types: vec![NumType::I32, NumType::I32],
                },
                results: ResultType {
                    value_types: vec![NumType::I32],
                },
            }],
        });

        assert_eq!(decoder.decode(), Ok(module));
    }
}