#[derive(Debug, PartialEq)]
struct Module {
    magic: [u8; 4],
    version: [u8; 4],
    type_section: Option<TypeSection>,
}

impl Module {
    fn new() -> Self {
        Self {
            magic: [0; 4],
            version: [0; 4],
            type_section: None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct TypeSection {

}

struct FunctionType {

}

struct ResultType {
    params: Vec<NumType>,
    results: Vec<NumType>,
}

enum NumType {
    I32,
    I64,
    F32,
    F64,
}

struct Decoder {
    input: Vec<u8>,
    pos: usize,
}

impl Decoder {
    fn new(input: Vec<u8>) -> Decoder {
        Self { input, pos: 0 }
    }

    fn decode(&mut self) -> Result<Module, ()> {
        let mut module = Module::new();
        module.magic = self.decode_magic_number();
        module.version = self.decode_version();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let buffer = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00].to_vec();
        let mut decorder = Decoder::new(buffer);

        let mut module = Module::new();
        module.magic = [0x00, 0x61, 0x73, 0x6d];
        module.version = [0x01, 0x00, 0x00, 0x00];

        assert_eq!(decorder.decode(), Ok(module));
    }
}