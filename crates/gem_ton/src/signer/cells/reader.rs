use primitives::SignerError;

#[derive(Clone, Debug)]
pub struct BitReader<'a> {
    bytes: &'a [u8],
    bit_len: usize,
    bit_position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_len: bytes.len() * 8,
            bit_position: 0,
        }
    }

    pub fn from_bits(bytes: &'a [u8], bit_len: usize) -> Result<Self, SignerError> {
        if bit_len > bytes.len() * 8 {
            return Err(SignerError::invalid_input("bit length exceeds input bytes"));
        }
        Ok(Self { bytes, bit_len, bit_position: 0 })
    }

    pub fn position(&self) -> usize {
        self.bit_position / 8
    }

    pub fn remaining(&self) -> usize {
        self.remaining_bits().div_ceil(8)
    }

    pub fn skip(&mut self, len: usize) -> Result<(), SignerError> {
        let _ = self.read_bytes(len)?;
        Ok(())
    }

    pub fn read_bit(&mut self) -> Result<bool, SignerError> {
        if self.bit_position >= self.bit_len {
            return Err(SignerError::invalid_input("unexpected end of input"));
        }

        let byte = self.bytes[self.bit_position / 8];
        let shift = 7 - (self.bit_position % 8);
        self.bit_position += 1;
        Ok((byte >> shift) & 1 == 1)
    }

    pub fn read_u8(&mut self) -> Result<u8, SignerError> {
        Ok(self.read_uint(8)? as u8)
    }

    pub fn read_u32(&mut self) -> Result<u32, SignerError> {
        Ok(self.read_uint(32)? as u32)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, SignerError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| SignerError::invalid_input("invalid u32"))?))
    }

    pub fn read_uint(&mut self, bit_len: usize) -> Result<u64, SignerError> {
        if bit_len > 64 {
            return Err(SignerError::invalid_input("fixed-size integers above 64 bits are not supported"));
        }

        let mut value = 0u64;
        for _ in 0..bit_len {
            value = (value << 1) | u64::from(self.read_bit()?);
        }
        Ok(value)
    }

    pub fn read_var_uint(&mut self, size: usize) -> Result<usize, SignerError> {
        let bytes = self.read_bytes(size)?;
        Ok(bytes.into_iter().fold(0usize, |acc, byte| (acc << 8) | byte as usize))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, SignerError> {
        if !self.bit_position.is_multiple_of(8) {
            return Err(SignerError::invalid_input("byte reads require byte-aligned position"));
        }

        let bit_len = len.checked_mul(8).ok_or_else(|| SignerError::invalid_input("invalid read length"))?;
        if self.remaining_bits() < bit_len {
            return Err(SignerError::invalid_input("unexpected end of input"));
        }

        let start = self.position();
        let end = start.checked_add(len).ok_or_else(|| SignerError::invalid_input("invalid read length"))?;
        self.bit_position += bit_len;
        Ok(self.bytes[start..end].to_vec())
    }

    fn remaining_bits(&self) -> usize {
        self.bit_len.saturating_sub(self.bit_position)
    }
}
