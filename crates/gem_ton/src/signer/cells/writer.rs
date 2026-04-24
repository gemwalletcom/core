use primitives::SignerError;

use super::cell::MAX_CELL_BITS;

#[derive(Default)]
pub struct BitWriter {
    pub bytes: Vec<u8>,
    pub bit_len: usize,
}

impl BitWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_bit(&mut self, value: bool) -> Result<(), SignerError> {
        if self.bit_len == MAX_CELL_BITS {
            return Err(SignerError::invalid_input(format!("cell exceeds {MAX_CELL_BITS} bits")));
        }
        if self.bit_len.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if value {
            let byte_index = self.bit_len / 8;
            let bit_index = 7 - (self.bit_len % 8);
            self.bytes[byte_index] |= 1 << bit_index;
        }
        self.bit_len += 1;
        Ok(())
    }

    pub fn write_uint(&mut self, bit_len: usize, value: u64) -> Result<(), SignerError> {
        if bit_len > 64 {
            return Err(SignerError::invalid_input("fixed-size integers above 64 bits are not supported"));
        }
        if bit_len < 64 && value >> bit_len != 0 {
            return Err(SignerError::invalid_input("integer does not fit requested bit length"));
        }
        for shift in (0..bit_len).rev() {
            self.write_bit((value >> shift) & 1 == 1)?;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SignerError> {
        self.write_bits(bytes, bytes.len() * 8)
    }

    pub fn write_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), SignerError> {
        if bit_len > bytes.len() * 8 {
            return Err(SignerError::invalid_input("bit length exceeds input bytes"));
        }
        for index in 0..bit_len {
            let byte = bytes[index / 8];
            let shift = 7 - (index % 8);
            self.write_bit((byte >> shift) & 1 == 1)?;
        }
        Ok(())
    }

}
