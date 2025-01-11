#[derive(Debug, PartialEq)]
pub struct Module {
    magic: [u8; 4],
    version: [u8; 4],
    type_section: Option<TypeSection>,
    function_section: Option<FunctionSection>,
    export_section: Option<ExportSection>,
    code_section: Option<CodeSection>,
}

impl Module {
    fn new() -> Self {
        Self {
            magic: [0; 4],
            version: [0; 4],
            type_section: None,
            function_section: None,
            export_section: None,
            code_section: None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct TypeSection {
    function_types: Vec<FunctionType>,
}

impl TypeSection {
    fn new() -> Self {
        Self { function_types: Vec::new() }
    }
}

#[derive(Debug, PartialEq)]
struct FunctionSection {
    type_idxs: Vec<u8>,
}

impl FunctionSection {
    fn new() -> Self { Self { type_idxs: Vec::new() } }
}

#[derive(Debug, PartialEq)]
struct ExportSection {
    exports: Vec<Export>,
}

impl ExportSection {
    fn new() -> Self { Self { exports: Vec::new() } }
}

#[derive(Debug, PartialEq)]
struct Export {
    name: Vec<u8>,
    export_desc: ExportDesc,
}

impl Export {
    fn new(
        name: Vec<u8>,
        export_desc: ExportDesc,
    ) -> Self {
        Self { name, export_desc }
    }
}

#[derive(Debug, PartialEq)]
enum ExportDesc {
    Func(u8),
    Table(u8),
    Mem(u8),
    Global(u8),
}

#[derive(Debug, PartialEq)]
struct CodeSection {
    codes: Vec<Code>,
}

impl CodeSection {
    fn new(codes: Vec<Code>) -> Self {
        Self { codes }
    }
}

#[derive(Debug, PartialEq)]
struct Code {
    instructions: Vec<Instruction>,
}

impl Code {
    fn new(instructions: Vec<Instruction>) -> Self { Self { instructions } }
}

#[derive(Debug, PartialEq)]
enum Instruction {
    LocalGet(u8),
    I32Add,
}


#[derive(Debug, PartialEq)]
struct FunctionType {
    params: ResultType,
    results: ResultType,
}

impl FunctionType {
    fn new() -> Self {
        Self { params: ResultType::new(), results: ResultType::new() }
    }
}

#[derive(Debug, PartialEq)]
struct ResultType {
    value_types: Vec<NumType>
}

impl ResultType {
    fn new() -> Self {
        Self { value_types: Vec::new() }
    }
}

#[derive(Debug, PartialEq)]
enum NumType {
    I32,
    I64,
    F32,
    F64,
}

pub struct Decoder {
    input: Vec<u8>,
    pos: usize,
}

impl Decoder {
    pub fn new(input: Vec<u8>) -> Decoder {
        Self { input, pos: 0 }
    }

    pub fn decode(&mut self) -> Result<Module, ()> {
        let mut module = Module::new();
        module.magic = self.decode_magic_number();
        module.version = self.decode_version();

        while self.pos < self.input.len() {
            let section_id = self.input[self.pos];
            self.pos += 1;

            match section_id {
                1 => {
                    module.type_section = Some(self.decode_type_section());
                }
                3 => {
                    module.function_section = Some(self.decode_function_section());
                }
                7 => {
                    module.export_section = Some(self.decode_export_section());
                }
                10 => {
                    module.code_section = Some(self.decode_code_section());
                }
                _ => {
                    let section_size = self.input[self.pos];
                    self.pos += 1;

                    self.pos += section_size as usize;

                    println!("id: {}, size: {}", section_id, section_size);
                }
            }
        }

        Ok(module)
    }

    fn decode_magic_number(&mut self) -> [u8; 4] {
        assert_eq!(self.pos, 0);

        let mut magic: [u8; 4] = [0; 4];
        for i in 0..4 {
            magic[i] = self.input[self.pos];
            self.pos += 1;
        }
        magic
    }

    fn decode_version(&mut self) -> [u8; 4] {
        assert_eq!(self.pos, 4);

        let mut version: [u8; 4] = [0; 4];
        for i in 0..4 {
            version[i] = self.input[self.pos];
            self.pos += 1;
        }
        version
    }

    fn decode_type_section(&mut self) -> TypeSection {
        let mut type_section = TypeSection::new();

        let section_size = self.input[self.pos];
        self.pos += 1;

        let num = self.input[self.pos];
        self.pos += 1;

        for i in 0..num {
            if let Some(function_type) = self.decode_function_type() {
                type_section.function_types.push(function_type);
            }
        }

        type_section
    }

    fn decode_function_type(&mut self) -> Option<FunctionType> {
        let byte = self.input[self.pos];
        self.pos += 1;

        if byte != 0x60 {
            return None;
        }

        let mut function_type = FunctionType::new();

        let num_of_params = self.input[self.pos];
        self.pos += 1;
        for _ in 0..num_of_params {
            let value_type = match self.input[self.pos] {
                0x7f => NumType::I32,
                _ => unimplemented!(),
            };
            function_type.params.value_types.push(value_type);
            self.pos += 1;
        }

        let num_of_results = self.input[self.pos];
        self.pos += 1;
        for _ in 0..num_of_results {
            let value_type = match self.input[self.pos] {
                0x7f => NumType::I32,
                _ => unimplemented!(),
            };
            function_type.results.value_types.push(value_type);
            self.pos += 1;
        }

        Some(function_type)
    }

    fn decode_function_section(&mut self) -> FunctionSection {
        let section_size = self.input[self.pos];
        self.pos += 1;

        let num = self.input[self.pos];
        self.pos += 1;

        let mut section = FunctionSection::new();

        for _ in 0..num {
            section.type_idxs.push(self.input[self.pos]);
            self.pos += 1;
        }

        section
    }

    fn decode_export_section(&mut self) -> ExportSection {
        let section_size = self.input[self.pos];
        self.pos += 1;

        let num = self.input[self.pos];
        self.pos += 1;

        let mut section = ExportSection::new();

        for _ in 0..num {
            let name_size = self.input[self.pos];
            self.pos += 1;

            let mut name = Vec::new();
            for _ in 0..name_size {
                name.push(self.input[self.pos]);
                self.pos += 1;
            }

            let desc_type = self.input[self.pos];
            self.pos += 1;

            let idx = self.input[self.pos];
            self.pos += 1;

            let export_desc = match desc_type {
                0x00 => ExportDesc::Func(idx),
                0x01 => ExportDesc::Table(idx),
                0x02 => ExportDesc::Mem(idx),
                0x03 => ExportDesc::Global(idx),
                _ => panic!(),
            };

            section.exports.push(Export::new(name, export_desc));
        }

        section
    }

    fn decode_code_section(&mut self) -> CodeSection {
        let section_size = self.input[self.pos];
        self.pos += 1;

        let num = self.input[self.pos];
        self.pos += 1;

        let mut codes = Vec::new();
        for _ in 0..num {
            let size = self.input[self.pos];
            self.pos += 1;

            let local = self.input[self.pos];
            self.pos += 1;

            let mut instructions = Vec::new();
            while self.pos < self.input.len() {
                if self.input[self.pos] == 0x0b {
                    self.pos += 1;
                    break;
                }

                let instruction_type = self.input[self.pos];
                self.pos += 1;
                let instruction = match instruction_type {
                    0x20 => {
                        let value = self.input[self.pos];
                        self.pos += 1;
                        Instruction::LocalGet(value)
                    },
                    0x6a => {
                        Instruction::I32Add
                    }
                    _ => unimplemented!(),
                };
                instructions.push(instruction);
            }
            codes.push(Code::new(instructions));
        }

        CodeSection::new(codes)
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