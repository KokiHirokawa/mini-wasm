use crate::structure::instructions::expression::Instr::If;
use crate::structure::instructions::expression::{BlockType, Expr, Instr};
use crate::structure::modules::export::{Export, ExportDesc};
use crate::structure::modules::function::Func;
use crate::structure::modules::indice::{FuncIdx, LocalIdx, TypeIdx};
use crate::structure::modules::module::Module;
use crate::structure::types::function::FuncType;
use crate::structure::types::value::NumType;
use crate::structure::types::value::ValType;
use std::iter::repeat_n;

#[derive(Debug)]
pub struct DecodingError {}

impl DecodingError {
    fn new() -> Self {
        Self {}
    }
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
                }
                3 => {
                    type_idxs = self.decode_function_section()?;
                }
                7 => {
                    module.exports = self.decode_export_section()?;
                }
                10 => {
                    module.funcs = self.decode_code_section(&type_idxs)?;
                }
                _ => {
                    let section_size = self.input[self.pos];
                    self.pos += 1;

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

        let section_size = self.decode_u32()?;

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

    fn decode_code_section(&mut self, type_idxs: &[TypeIdx]) -> Result<Vec<Func>, DecodingError> {
        let section_size = self.decode_u32()?;

        let num_of_funcs = self.decode_u32()?;

        let mut funcs = Vec::new();
        for i in 0..num_of_funcs {
            let size = self.decode_u32()?;
            let num_of_locals = self.decode_u32()?;

            let mut locals = Vec::new();
            for _ in 0..num_of_locals {
                let n = self.decode_u32()?;

                let val_type = match self.input[self.pos] {
                    0x7f => ValType::NumType(NumType::I32),
                    0x7e => ValType::NumType(NumType::I64),
                    0x7d => ValType::NumType(NumType::F32),
                    0x7c => ValType::NumType(NumType::F64),
                    _ => unimplemented!("unimplemented value type"),
                };
                self.pos += 1;

                locals.extend(repeat_n(val_type, n as usize));
            }

            let mut body = Expr(Vec::new());
            loop {
                let instr = self.decode_instruction()?;
                if instr == Instr::End {
                    break;
                }
                body.0.push(instr);
            }
            println!("{:?}", body);
            let func = Func {
                type_: type_idxs[i as usize],
                locals,
                body,
            };
            funcs.push(func);
        }

        Ok(funcs)
    }

    fn decode_instruction(&mut self) -> Result<Instr, DecodingError> {
        let byte = self.input[self.pos];
        self.pos += 1;

        let instr = match byte {
            0x04 => {
                let block_type = match self.input[self.pos] {
                    0x7f => BlockType::ValType(ValType::NumType(NumType::I32)),
                    0x7e => BlockType::ValType(ValType::NumType(NumType::I64)),
                    0x7d => BlockType::ValType(ValType::NumType(NumType::F32)),
                    0x7c => BlockType::ValType(ValType::NumType(NumType::F64)),
                    _ => unimplemented!("unimplemented block type"),
                };
                self.pos += 1;

                let mut instructions1 = Vec::new();
                let mut instructions2 = Vec::new();
                'outer: loop {
                    let instr = self.decode_instruction()?;

                    if instr == Instr::Else {
                        loop {
                            let instr = self.decode_instruction()?;

                            if instr == Instr::End {
                                break 'outer;
                            } else {
                                instructions2.push(instr);
                            }
                        }
                    } else {
                        instructions1.push(instr);
                    }
                }

                If(block_type, instructions1, instructions2)
            }
            0x05 => Instr::Else,
            0x0b => Instr::End,
            0x10 => {
                let idx = self.decode_u32()?;
                Instr::Call(idx)
            }
            0x1A => Instr::Drop,
            0x20 => {
                let idx = self.decode_u32()?;
                Instr::LocalGet(idx as LocalIdx)
            }
            0x41 => {
                let x = self.decode_u32()? as i32; // FIXME: decode_i32
                Instr::I32Const(x)
            }
            0x42 => {
                let x = self.decode_i64()?;
                Instr::I64Const(x)
            }
            0x45 => Instr::I32Eqz,
            0x46 => Instr::I32Eq,
            0x47 => Instr::I32Ne,
            0x48 => Instr::I32LtS,
            0x49 => Instr::I32LtU,
            0x4a => Instr::I32GtS,
            0x4b => Instr::I32GtU,
            0x4c => Instr::I32LeS,
            0x4d => Instr::I32LeU,
            0x4e => Instr::I32GeS,
            0x4f => Instr::I32GeU,
            0x50 => Instr::I64Eqz,
            0x51 => Instr::I64Eq,
            0x52 => Instr::I64Ne,
            0x53 => Instr::I64LtS,
            0x54 => Instr::I64LtU,
            0x55 => Instr::I64GtS,
            0x56 => Instr::I64GtU,
            0x57 => Instr::I64LeS,
            0x58 => Instr::I64LeU,
            0x59 => Instr::I64GeS,
            0x5a => Instr::I64GeU,
            0x5b => Instr::F32Eq,
            0x5c => Instr::F32Ne,
            0x5d => Instr::F32Lt,
            0x5e => Instr::F32Gt,
            0x5f => Instr::F32Le,
            0x60 => Instr::F32Ge,
            0x61 => Instr::F64Eq,
            0x62 => Instr::F64Ne,
            0x63 => Instr::F64Lt,
            0x64 => Instr::F64Gt,
            0x65 => Instr::F64Le,
            0x66 => Instr::F64Ge,
            0x67 => Instr::I32Clz,
            0x68 => Instr::I32Ctz,
            0x69 => Instr::I32Popcnt,
            0x6a => Instr::I32Add,
            0x6b => Instr::I32Sub,
            0x6c => Instr::I32Mul,
            0x6d => Instr::I32DivS,
            0x6e => Instr::I32DivU,
            0x6f => Instr::I32RemS,
            0x70 => Instr::I32RemU,
            0x71 => Instr::I32And,
            0x72 => Instr::I32Or,
            0x73 => Instr::I32Xor,
            0x74 => Instr::I32Shl,
            0x75 => Instr::I32ShrS,
            0x76 => Instr::I32ShrU,
            0x77 => Instr::I32Rotl,
            0x78 => Instr::I32Rotr,
            0x79 => Instr::I64Clz,
            0x7a => Instr::I64Ctz,
            0x7b => Instr::I64Popcnt,
            0x7c => Instr::I64Add,
            0x7d => Instr::I64Sub,
            0x7e => Instr::I64Mul,
            0x7f => Instr::I64DivS,
            0x80 => Instr::I64DivU,
            0x81 => Instr::I64RemS,
            0x82 => Instr::I64RemU,
            0x83 => Instr::I64And,
            0x84 => Instr::I64Or,
            0x85 => Instr::I64Xor,
            0x86 => Instr::I64Shl,
            0x87 => Instr::I64ShrS,
            0x88 => Instr::I64ShrU,
            0x89 => Instr::I64Rotl,
            0x8a => Instr::I64Rotr,
            0x8b => Instr::F32Abs,
            0x8c => Instr::F32Neg,
            0x8d => Instr::F32Ceil,
            0x8e => Instr::F32Floor,
            0x8f => Instr::F32Trunc,
            0x90 => Instr::F32Nearest,
            0x91 => Instr::F32Sqrt,
            0x92 => Instr::F32Add,
            0x93 => Instr::F32Sub,
            0x94 => Instr::F32Mul,
            0x95 => Instr::F32Div,
            0x96 => Instr::F32Min,
            0x97 => Instr::F32Max,
            0x98 => Instr::F32Copysign,
            0x99 => Instr::F64Abs,
            0x9a => Instr::F64Neg,
            0x9b => Instr::F64Ceil,
            0x9c => Instr::F64Floor,
            0x9d => Instr::F64Trunc,
            0x9e => Instr::F64Nearest,
            0x9f => Instr::F64Sqrt,
            0xa0 => Instr::F64Add,
            0xa1 => Instr::F64Sub,
            0xa2 => Instr::F64Mul,
            0xa3 => Instr::F64Div,
            0xa4 => Instr::F64Min,
            0xa5 => Instr::F64Max,
            0xa6 => Instr::F64Copysign,
            0xc0 => Instr::I32Extend8S,
            0xc1 => Instr::I32Extend16S,
            _ => unimplemented!("unimplemented instr {:#x}", byte),
        };
        Ok(instr)
    }

    fn decode_i64(&mut self) -> Result<i64, DecodingError> {
        let mut result: i64 = 0;
        let mut shift = 0;

        loop {
            let byte = self.input[self.pos];
            self.pos += 1;

            let value = (byte & 0b01111111) as i64;
            result |= value << shift;
            shift += 7;

            if (byte & 0b10000000) == 0 {
                if shift < 64 && byte & 0x40 != 0 {
                    result |= !0 << shift;
                }
                break;
            }
        }

        Ok(result)
    }

    fn decode_u32(&mut self) -> Result<u32, DecodingError> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.input[self.pos];
            self.pos += 1;

            let value = (byte & 0b01111111) as u32;
            result |= value << shift;
            shift += 7;

            if (byte & 0b10000000) == 0 {
                break;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    fn decode(filename: &str) -> Result<Module, DecodingError> {
        let mut file = File::open(format!("./tests/inputs/{}.wasm", filename)).unwrap();
        let mut input = Vec::new();
        file.read_to_end(&mut input).unwrap();
        let mut decoder = Decoder::new(&input);
        decoder.decode()
    }

    #[test]
    fn test_i32_add() {
        let module = decode("i32.add").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::I32)
                ],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "add".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32_div_s() {
        let module = decode("i32.div_s").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::I32)
                ],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32DivS]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "div_s".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32_clz() {
        let module = decode("i32.clz").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::I32Clz]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "clz".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32_eq() {
        let module = decode("i32.eq").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::I32)
                ],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Eq]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "eq".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32_eqz() {
        let module = decode("i32.eqz").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::I32Eqz]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "eqz".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32_extend8_s() {
        let module = decode("i32.extend8_s").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![ValType::NumType(NumType::I32)],
                results: vec![ValType::NumType(NumType::I32)],
            }],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: Vec::new(),
                body: Expr(vec![Instr::LocalGet(0), Instr::I32Extend8S]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "extend8_s".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }

    #[test]
    fn test_i32() {
        let module = decode("i32").unwrap();

        assert_eq!(
            module.types,
            vec![
                FuncType {
                    parameters: vec![
                        ValType::NumType(NumType::I32),
                        ValType::NumType(NumType::I32)
                    ],
                    results: vec![ValType::NumType(NumType::I32)],
                },
                FuncType {
                    parameters: vec![ValType::NumType(NumType::I32)],
                    results: vec![ValType::NumType(NumType::I32)],
                },
            ],
        );
        assert_eq!(
            module.funcs,
            vec![
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Add]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Sub]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Mul]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32DivS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32DivU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32RemS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32RemU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32And]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Or]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Xor]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Shl]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32ShrS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32ShrU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Rotl]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Rotr]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Clz]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Ctz]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Popcnt]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Extend8S]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Extend16S]),
                },
                Func {
                    type_: 1,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::I32Eqz]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Eq]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32Ne]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32LtS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32LtU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32LeS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32LeU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32GtS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32GtU]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32GeS]),
                },
                Func {
                    type_: 0,
                    locals: Vec::new(),
                    body: Expr(vec![Instr::LocalGet(0), Instr::LocalGet(1), Instr::I32GeU]),
                },
            ],
        );
        assert_eq!(
            module.exports,
            vec![
                Export {
                    name: "add".to_string(),
                    desc: ExportDesc::Func(0),
                },
                Export {
                    name: "sub".to_string(),
                    desc: ExportDesc::Func(1),
                },
                Export {
                    name: "mul".to_string(),
                    desc: ExportDesc::Func(2),
                },
                Export {
                    name: "div_s".to_string(),
                    desc: ExportDesc::Func(3),
                },
                Export {
                    name: "div_u".to_string(),
                    desc: ExportDesc::Func(4),
                },
                Export {
                    name: "rem_s".to_string(),
                    desc: ExportDesc::Func(5),
                },
                Export {
                    name: "rem_u".to_string(),
                    desc: ExportDesc::Func(6),
                },
                Export {
                    name: "and".to_string(),
                    desc: ExportDesc::Func(7),
                },
                Export {
                    name: "or".to_string(),
                    desc: ExportDesc::Func(8),
                },
                Export {
                    name: "xor".to_string(),
                    desc: ExportDesc::Func(9),
                },
                Export {
                    name: "shl".to_string(),
                    desc: ExportDesc::Func(10),
                },
                Export {
                    name: "shr_s".to_string(),
                    desc: ExportDesc::Func(11),
                },
                Export {
                    name: "shr_u".to_string(),
                    desc: ExportDesc::Func(12),
                },
                Export {
                    name: "rotl".to_string(),
                    desc: ExportDesc::Func(13),
                },
                Export {
                    name: "rotr".to_string(),
                    desc: ExportDesc::Func(14),
                },
                Export {
                    name: "clz".to_string(),
                    desc: ExportDesc::Func(15),
                },
                Export {
                    name: "ctz".to_string(),
                    desc: ExportDesc::Func(16),
                },
                Export {
                    name: "popcnt".to_string(),
                    desc: ExportDesc::Func(17),
                },
                Export {
                    name: "extend8_s".to_string(),
                    desc: ExportDesc::Func(18),
                },
                Export {
                    name: "extend16_s".to_string(),
                    desc: ExportDesc::Func(19),
                },
                Export {
                    name: "eqz".to_string(),
                    desc: ExportDesc::Func(20),
                },
                Export {
                    name: "eq".to_string(),
                    desc: ExportDesc::Func(21),
                },
                Export {
                    name: "ne".to_string(),
                    desc: ExportDesc::Func(22),
                },
                Export {
                    name: "lt_s".to_string(),
                    desc: ExportDesc::Func(23),
                },
                Export {
                    name: "lt_u".to_string(),
                    desc: ExportDesc::Func(24),
                },
                Export {
                    name: "le_s".to_string(),
                    desc: ExportDesc::Func(25),
                },
                Export {
                    name: "le_u".to_string(),
                    desc: ExportDesc::Func(26),
                },
                Export {
                    name: "gt_s".to_string(),
                    desc: ExportDesc::Func(27),
                },
                Export {
                    name: "gt_u".to_string(),
                    desc: ExportDesc::Func(28),
                },
                Export {
                    name: "ge_s".to_string(),
                    desc: ExportDesc::Func(29),
                },
                Export {
                    name: "ge_u".to_string(),
                    desc: ExportDesc::Func(30),
                },
            ],
        );
    }

    #[test]
    fn test_local_get_0() {
        let module = decode("local_get.0").unwrap();

        assert_eq!(
            module.types,
            vec![
                FuncType {
                    parameters: vec![],
                    results: vec![ValType::NumType(NumType::I32)],
                },
                FuncType {
                    parameters: vec![],
                    results: vec![ValType::NumType(NumType::I64)],
                },
                FuncType {
                    parameters: vec![],
                    results: vec![ValType::NumType(NumType::F32)],
                },
                FuncType {
                    parameters: vec![],
                    results: vec![ValType::NumType(NumType::F64)],
                },
            ],
        );
        assert_eq!(
            module.funcs,
            vec![
                Func {
                    type_: 0,
                    locals: vec![ValType::NumType(NumType::I32)],
                    body: Expr(vec![Instr::LocalGet(0)]),
                },
                Func {
                    type_: 1,
                    locals: vec![ValType::NumType(NumType::I64)],
                    body: Expr(vec![Instr::LocalGet(0)]),
                },
                Func {
                    type_: 2,
                    locals: vec![ValType::NumType(NumType::F32)],
                    body: Expr(vec![Instr::LocalGet(0)]),
                },
                Func {
                    type_: 3,
                    locals: vec![ValType::NumType(NumType::F64)],
                    body: Expr(vec![Instr::LocalGet(0)]),
                },
            ],
        );
        assert_eq!(
            module.exports,
            vec![
                Export {
                    name: "type-local-i32".to_string(),
                    desc: ExportDesc::Func(0),
                },
                Export {
                    name: "type-local-i64".to_string(),
                    desc: ExportDesc::Func(1),
                },
                Export {
                    name: "type-local-f32".to_string(),
                    desc: ExportDesc::Func(2),
                },
                Export {
                    name: "type-local-f64".to_string(),
                    desc: ExportDesc::Func(3),
                },
            ],
        );
    }

    #[test]
    fn test_local_get_1() {
        let module = decode("local_get.1").unwrap();

        assert_eq!(
            module.types,
            vec![FuncType {
                parameters: vec![
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::F64),
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::I32),
                ],
                results: vec![],
            },],
        );
        assert_eq!(
            module.funcs,
            vec![Func {
                type_: 0,
                locals: vec![
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::F64),
                ],
                body: Expr(vec![
                    Instr::LocalGet(0),
                    Instr::I64Eqz,
                    Instr::Drop,
                    Instr::LocalGet(1),
                    Instr::F32Neg,
                    Instr::Drop,
                    Instr::LocalGet(2),
                    Instr::F64Neg,
                    Instr::Drop,
                    Instr::LocalGet(3),
                    Instr::I32Eqz,
                    Instr::Drop,
                    Instr::LocalGet(4),
                    Instr::I32Eqz,
                    Instr::Drop,
                    Instr::LocalGet(5),
                    Instr::F32Neg,
                    Instr::Drop,
                    Instr::LocalGet(6),
                    Instr::I64Eqz,
                    Instr::Drop,
                    Instr::LocalGet(7),
                    Instr::I64Eqz,
                    Instr::Drop,
                    Instr::LocalGet(8),
                    Instr::F64Neg,
                    Instr::Drop,
                ]),
            }],
        );
        assert_eq!(
            module.exports,
            vec![Export {
                name: "type-mixed".to_string(),
                desc: ExportDesc::Func(0),
            }],
        );
    }
}
