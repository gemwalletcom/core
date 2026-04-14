use std::sync::Arc;

use gem_hash::sha2::sha256;
use primitives::SignerError;

pub(super) const MAX_CELL_BITS: usize = 1023;
pub(super) const MAX_CELL_REFERENCES: usize = 4;

pub type CellArc = Arc<Cell>;

#[derive(Clone, Debug)]
pub struct Cell {
    data: Vec<u8>,
    bit_len: usize,
    references: Vec<CellArc>,
    depth: u16,
    hash: [u8; 32],
}

impl Cell {
    pub fn new(data: Vec<u8>, bit_len: usize, references: Vec<CellArc>) -> Result<Self, SignerError> {
        if bit_len > MAX_CELL_BITS {
            return Err(SignerError::invalid_input(format!("cell exceeds {MAX_CELL_BITS} bits")));
        }
        if references.len() > MAX_CELL_REFERENCES {
            return Err(SignerError::invalid_input(format!("cell exceeds {MAX_CELL_REFERENCES} references")));
        }
        if data.len() != bit_len.div_ceil(8) {
            return Err(SignerError::invalid_input("cell data length does not match bit length"));
        }

        let depth = if references.is_empty() {
            0
        } else {
            let max_depth = references.iter().map(|reference| reference.depth).max().unwrap_or_default();
            max_depth.checked_add(1).ok_or_else(|| SignerError::invalid_input("cell depth overflow"))?
        };

        let mut repr = Vec::with_capacity(2 + data.len() + references.len() * 34);
        repr.push(references.len() as u8);
        repr.push(bits_descriptor(bit_len)?);
        repr.extend_from_slice(&serialized_bits(&data, bit_len));
        for reference in &references {
            repr.extend_from_slice(&reference.depth.to_be_bytes());
        }
        for reference in &references {
            repr.extend_from_slice(&reference.hash);
        }

        Ok(Self {
            data,
            bit_len,
            references,
            depth,
            hash: sha256(&repr),
        })
    }

    pub fn into_arc(self) -> CellArc {
        Arc::new(self)
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn references(&self) -> &[CellArc] {
        &self.references
    }

    pub fn cell_hash(&self) -> [u8; 32] {
        self.hash
    }
}

pub(super) fn bits_descriptor(bit_len: usize) -> Result<u8, SignerError> {
    let data_len = bit_len.div_ceil(8);
    if data_len > 128 {
        return Err(SignerError::invalid_input("cell payload too large"));
    }
    Ok((data_len * 2 - usize::from(!bit_len.is_multiple_of(8))) as u8)
}

pub(super) fn serialized_bits(data: &[u8], bit_len: usize) -> Vec<u8> {
    let data_len = bit_len.div_ceil(8);
    if data_len == 0 {
        return Vec::new();
    }

    let mut serialized = data[..data_len].to_vec();
    if !bit_len.is_multiple_of(8) {
        let marker_shift = 8 - (bit_len % 8) - 1;
        let last_index = serialized.len() - 1;
        serialized[last_index] |= 1 << marker_shift;
    }
    serialized
}
