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

        let section_size = self.decode_u32()?;
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
        let section_size = self.decode_u32()?;
        println!("size of code section: {}", section_size);

        let num_of_funcs = self.decode_u32()?;

        let mut funcs = Vec::new();
        for i in 0..num_of_funcs {
            let size = self.decode_u32()?;
            let num_of_locals = self.decode_u32()?;

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
                    0xc0 => Instr::I32Extend8S,
                    0xc1 => Instr::I32Extend16S,
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

    fn decode_u32(&mut self) -> Result<u32, DecodingError> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.input[self.pos];
            self.pos += 1;

            let value = (byte & 0x7f) as u32;
            result += value << shift;

            if (byte & 0x80) == 0 {
                break;
            }

            shift += 7;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use super::*;

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
                parameters: vec![ValType::NumType(NumType::I32), ValType::NumType(NumType::I32)],
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
                parameters: vec![ValType::NumType(NumType::I32), ValType::NumType(NumType::I32)],
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
                parameters: vec![ValType::NumType(NumType::I32), ValType::NumType(NumType::I32)],
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
                    parameters: vec![ValType::NumType(NumType::I32), ValType::NumType(NumType::I32)],
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
}