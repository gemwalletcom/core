use std::sync::Arc;

use gem_hash::sha2::sha256;
use primitives::SignerError;

pub(super) const MAX_CELL_BITS: usize = 1023;
pub(super) const MAX_CELL_REFERENCES: usize = 4;

pub type CellArc = Arc<Cell>;

#[derive(Clone, Debug)]
pub struct Cell {
    pub data: Vec<u8>,
    pub bit_len: usize,
    pub references: Vec<CellArc>,
    pub(super) depth: u16,
    pub hash: [u8; 32],
}

impl Cell {
    pub fn try_new(data: Vec<u8>, bit_len: usize, references: Vec<CellArc>) -> Option<Self> {
        if bit_len > MAX_CELL_BITS || references.len() > MAX_CELL_REFERENCES || data.len() != bit_len.div_ceil(8) {
            return None;
        }

        let depth = if references.is_empty() {
            0
        } else {
            references.iter().map(|reference| reference.depth).max()?.checked_add(1)?
        };

        let mut repr = Vec::with_capacity(2 + data.len() + references.len() * 34);
        repr.push(references.len() as u8);
        repr.push(Self::bits_descriptor(bit_len)?);
        repr.extend_from_slice(&Self::serialized_bits(&data, bit_len));
        repr.extend(references.iter().flat_map(|reference| reference.depth.to_be_bytes()));
        repr.extend(references.iter().flat_map(|reference| reference.hash));

        Some(Self {
            data,
            bit_len,
            references,
            depth,
            hash: sha256(&repr),
        })
    }

    pub fn new(data: Vec<u8>, bit_len: usize, references: Vec<CellArc>) -> Result<Self, SignerError> {
        Self::try_new(data, bit_len, references).ok_or_else(|| SignerError::invalid_input("invalid cell"))
    }

    pub fn into_arc(self) -> CellArc {
        Arc::new(self)
    }

    fn bits_descriptor(bit_len: usize) -> Option<u8> {
        let data_len = bit_len.div_ceil(8);
        if data_len > 128 {
            return None;
        }
        Some((data_len * 2 - usize::from(!bit_len.is_multiple_of(8))) as u8)
    }

    fn serialized_bits(data: &[u8], bit_len: usize) -> Vec<u8> {
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
}
