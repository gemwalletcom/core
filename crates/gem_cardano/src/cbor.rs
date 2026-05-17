pub(crate) struct CborEncoder {
    bytes: Vec<u8>,
}

impl CborEncoder {
    pub(crate) fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub(crate) fn raw(&mut self, raw: &[u8]) {
        self.bytes.extend_from_slice(raw);
    }

    pub(crate) fn unsigned(&mut self, value: u64) {
        self.major_type(0, value);
    }

    pub(crate) fn bytes(&mut self, value: &[u8]) {
        self.major_type(2, value.len() as u64);
        self.bytes.extend_from_slice(value);
    }

    pub(crate) fn array(&mut self, len: usize) {
        self.major_type(4, len as u64);
    }

    pub(crate) fn map(&mut self, len: usize) {
        self.major_type(5, len as u64);
    }

    pub(crate) fn null(&mut self) {
        self.bytes.push(0xf6);
    }

    pub(crate) fn true_value(&mut self) {
        self.bytes.push(0xf5);
    }

    fn major_type(&mut self, major: u8, value: u64) {
        let prefix = major << 5;
        match value {
            0..=23 => self.bytes.push(prefix | value as u8),
            24..=0xff => {
                self.bytes.push(prefix | 24);
                self.bytes.push(value as u8);
            }
            0x100..=0xffff => {
                self.bytes.push(prefix | 25);
                self.bytes.extend_from_slice(&(value as u16).to_be_bytes());
            }
            0x1_0000..=0xffff_ffff => {
                self.bytes.push(prefix | 26);
                self.bytes.extend_from_slice(&(value as u32).to_be_bytes());
            }
            _ => {
                self.bytes.push(prefix | 27);
                self.bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
}
