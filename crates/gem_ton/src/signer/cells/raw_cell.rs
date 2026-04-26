use primitives::SignerError;

use super::{cell::Cell, indexed_cell::IndexedCells, invalid, reader::BitReader};

#[derive(Clone)]
pub(super) struct RawCell {
    pub data: Vec<u8>,
    pub bit_len: usize,
    pub references: Vec<usize>,
}

impl RawCell {
    pub(super) fn try_parse(reader: &mut BitReader<'_>, ref_size: usize) -> Option<Self> {
        // d1 (refs descriptor): bits[0..3] ref count | bit 3 is_exotic | bit 4 has_hashes | bits[5..7] level mask
        let refs_descriptor = reader.read_u8().ok()?;
        // d2 (bits descriptor): encodes data length; LSB=0 means full bytes, LSB=1 means last byte has padding
        let bits_descriptor = reader.read_u8().ok()?;

        let ref_count = (refs_descriptor & 0b111) as usize;
        let is_exotic = refs_descriptor & 0b1000 != 0;
        let has_hashes = refs_descriptor & 0b10000 != 0;
        let level_mask = refs_descriptor >> 5;
        if is_exotic || has_hashes || level_mask != 0 {
            return None;
        }

        let data_size = ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize;
        let full_bytes = bits_descriptor & 1 == 0;
        let data = reader.read_bytes(data_size).ok()?;
        let (data, bit_len) = Self::unpad_cell_bits(data, full_bytes)?;

        let references = (0..ref_count).map(|_| reader.read_var_uint(ref_size).ok()).collect::<Option<Vec<_>>>()?;
        Some(Self { data, bit_len, references })
    }

    pub(super) fn parse(reader: &mut BitReader<'_>, ref_size: usize) -> Result<Self, SignerError> {
        Self::try_parse(reader, ref_size).ok_or_else(|| invalid("invalid BoC cell"))
    }

    pub(super) fn from_cell(cell: &Cell, indexed_cells: &IndexedCells) -> Result<Self, SignerError> {
        let references = cell
            .references
            .iter()
            .map(|reference| {
                indexed_cells
                    .get(&reference.hash)
                    .map(|indexed| indexed.borrow().index)
                    .ok_or_else(|| invalid("missing referenced cell"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            data: cell.data.clone(),
            bit_len: cell.bit_len,
            references,
        })
    }

    pub(super) fn size(&self, ref_size: usize) -> usize {
        2 + self.bit_len.div_ceil(8) + self.references.len() * ref_size
    }

    pub(super) fn write(&self, output: &mut Vec<u8>, ref_size: usize) {
        let full_bytes = self.bit_len.is_multiple_of(8);
        let data_len = self.bit_len.div_ceil(8);
        output.push(self.references.len() as u8);
        output.push((data_len * 2 - usize::from(!full_bytes)) as u8);

        if !full_bytes && data_len > 0 {
            output.extend_from_slice(&self.data[..data_len - 1]);
            let padding_bits = self.bit_len % 8;
            output.push(self.data[data_len - 1] | (1 << (8 - padding_bits - 1)));
        } else {
            output.extend_from_slice(&self.data[..data_len]);
        }

        for reference in &self.references {
            write_var_uint(output, *reference, ref_size);
        }
    }

    fn unpad_cell_bits(mut data: Vec<u8>, full_bytes: bool) -> Option<(Vec<u8>, usize)> {
        if data.is_empty() {
            return Some((data, 0));
        }
        if full_bytes {
            let bit_len = data.len() * 8;
            return Some((data, bit_len));
        }

        let trailing_zeros = data.last().copied().unwrap_or_default().trailing_zeros();
        if trailing_zeros >= 8 {
            return None;
        }
        let last = data.last_mut()?;
        *last &= !(1 << trailing_zeros);
        let bit_len = data.len() * 8 - (trailing_zeros as usize + 1);
        Some((data, bit_len))
    }
}

pub(super) fn write_var_uint(output: &mut Vec<u8>, value: usize, size: usize) {
    for shift in (0..size).rev() {
        output.push(((value >> (shift * 8)) & 0xff) as u8);
    }
}
