use num_bigint::BigUint;
use primitives::SignerError;

use super::{
    cell::{Cell, CellArc, MAX_CELL_BITS, MAX_CELL_REFERENCES},
    writer::BitWriter,
};
use crate::address::Address;

#[derive(Default)]
pub struct CellBuilder {
    writer: BitWriter,
    references: Vec<CellArc>,
}

impl CellBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store_bit(&mut self, value: bool) -> Result<&mut Self, SignerError> {
        self.writer.write_bit(value)?;
        Ok(self)
    }

    pub fn store_u8(&mut self, bit_len: usize, value: u8) -> Result<&mut Self, SignerError> {
        self.writer.write_uint(bit_len, value as u64)?;
        Ok(self)
    }

    pub fn store_u32(&mut self, bit_len: usize, value: u32) -> Result<&mut Self, SignerError> {
        self.writer.write_uint(bit_len, value as u64)?;
        Ok(self)
    }

    pub fn store_i32(&mut self, bit_len: usize, value: i32) -> Result<&mut Self, SignerError> {
        self.writer.write_uint(bit_len, value as u32 as u64)?;
        Ok(self)
    }

    pub fn store_u64(&mut self, bit_len: usize, value: u64) -> Result<&mut Self, SignerError> {
        self.writer.write_uint(bit_len, value)?;
        Ok(self)
    }

    pub fn store_slice(&mut self, slice: &[u8]) -> Result<&mut Self, SignerError> {
        self.writer.write_bytes(slice)?;
        Ok(self)
    }

    pub fn store_slice_snake(&mut self, slice: &[u8]) -> Result<&mut Self, SignerError> {
        let byte_capacity = self.remaining_bits() / 8;
        if slice.len() <= byte_capacity {
            return self.store_slice(slice);
        }

        let (head, tail) = slice.split_at(byte_capacity);
        self.store_slice(head)?;

        let mut child = Self::new();
        child.store_slice_snake(tail)?;
        self.store_child(child.build()?)?;
        Ok(self)
    }

    pub fn store_string_snake(&mut self, value: &str) -> Result<&mut Self, SignerError> {
        self.store_slice_snake(value.as_bytes())
    }

    pub fn store_uint(&mut self, bit_len: usize, value: &BigUint) -> Result<&mut Self, SignerError> {
        let used_bits = biguint_bit_len(value);
        if used_bits > bit_len {
            return Err(SignerError::invalid_input(format!("value does not fit in {bit_len} bits")));
        }

        let leading_zero_bits = bit_len.saturating_sub(used_bits);
        let leading_zero_bytes = leading_zero_bits / 8;
        for _ in 0..leading_zero_bytes {
            self.store_u8(8, 0)?;
        }

        let extra_zero_bits = leading_zero_bits % 8;
        for _ in 0..extra_zero_bits {
            self.store_bit(false)?;
        }

        let bytes = value.to_bytes_be();
        if let Some(high_byte) = bytes.first() {
            let high_bits = if used_bits == 0 {
                0
            } else {
                let bits = used_bits % 8;
                if bits == 0 { 8 } else { bits }
            };
            if high_bits > 0 {
                for shift in (0..high_bits).rev() {
                    self.store_bit((high_byte >> shift) & 1 == 1)?;
                }
            }
            for byte in bytes.iter().skip(1) {
                self.store_u8(8, *byte)?;
            }
        }

        Ok(self)
    }

    pub fn store_coins(&mut self, value: &BigUint) -> Result<&mut Self, SignerError> {
        if value == &BigUint::from(0u8) {
            self.store_u8(4, 0)?;
            return Ok(self);
        }

        let bytes = value.to_bytes_be();
        self.store_u8(4, bytes.len() as u8)?;
        self.store_uint(bytes.len() * 8, value)
    }

    pub fn store_null_address(&mut self) -> Result<&mut Self, SignerError> {
        self.store_u8(2, 0)
    }

    pub fn store_address(&mut self, address: &Address) -> Result<&mut Self, SignerError> {
        self.store_u8(2, 0b10)?;
        self.store_bit(false)?;
        self.store_u8(8, address.workchain() as i8 as u8)?;
        self.store_slice(address.get_hash_part())?;
        Ok(self)
    }

    pub fn store_reference(&mut self, cell: &CellArc) -> Result<&mut Self, SignerError> {
        let next_len = self.references.len() + 1;
        if next_len > MAX_CELL_REFERENCES {
            return Err(SignerError::invalid_input(format!("cell exceeds {MAX_CELL_REFERENCES} references")));
        }
        self.references.push(cell.clone());
        Ok(self)
    }

    pub fn store_child(&mut self, cell: Cell) -> Result<&mut Self, SignerError> {
        self.store_reference(&cell.into_arc())
    }

    pub fn store_cell_data(&mut self, cell: &Cell) -> Result<&mut Self, SignerError> {
        self.writer.write_bits(&cell.data, cell.bit_len)?;
        Ok(self)
    }

    pub fn store_cell(&mut self, cell: &Cell) -> Result<&mut Self, SignerError> {
        self.store_cell_data(cell)?;
        for reference in &cell.references {
            self.store_reference(reference)?;
        }
        Ok(self)
    }

    pub fn remaining_bits(&self) -> usize {
        MAX_CELL_BITS.saturating_sub(self.writer.bit_len())
    }

    pub fn build(self) -> Result<Cell, SignerError> {
        let bit_len = self.writer.bit_len();
        let bytes = self.writer.finish_bytes();
        Cell::new(bytes, bit_len, self.references)
    }
}

fn biguint_bit_len(value: &BigUint) -> usize {
    let bytes = value.to_bytes_be();
    match bytes.first() {
        Some(first) => (bytes.len() - 1) * 8 + (8 - first.leading_zeros() as usize),
        None => 0,
    }
}
