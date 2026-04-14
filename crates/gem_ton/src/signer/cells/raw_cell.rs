use primitives::SignerError;

use super::cell::Cell;
use super::indexed_cell::IndexedCells;
use super::invalid;
use super::reader::BitReader;

#[derive(Clone)]
pub(super) struct RawCell {
    pub data: Vec<u8>,
    pub bit_len: usize,
    pub references: Vec<usize>,
}

pub(super) fn read_raw_cell(reader: &mut BitReader<'_>, ref_size: usize) -> Result<RawCell, SignerError> {
    let d1 = reader.read_u8()?;
    let d2 = reader.read_u8()?;

    let ref_count = (d1 & 0b111) as usize;
    let is_exotic = d1 & 0b1000 != 0;
    let has_hashes = d1 & 0b10000 != 0;
    let level_mask = d1 >> 5;
    if is_exotic || has_hashes || level_mask != 0 {
        return Err(invalid("unsupported exotic or hashed BoC cell"));
    }

    let data_size = ((d2 >> 1) + (d2 & 1)) as usize;
    let full_bytes = d2 & 1 == 0;
    let data = reader.read_bytes(data_size)?;
    let (data, bit_len) = unpad_cell_bits(data, full_bytes)?;

    let references = (0..ref_count).map(|_| reader.read_var_uint(ref_size)).collect::<Result<Vec<_>, _>>()?;
    Ok(RawCell { data, bit_len, references })
}

fn unpad_cell_bits(mut data: Vec<u8>, full_bytes: bool) -> Result<(Vec<u8>, usize), SignerError> {
    if data.is_empty() {
        return Ok((data, 0));
    }
    if full_bytes {
        let bit_len = data.len() * 8;
        return Ok((data, bit_len));
    }

    let trailing_zeros = data.last().copied().unwrap_or_default().trailing_zeros();
    if trailing_zeros >= 8 {
        return Err(invalid("invalid padded BoC cell"));
    }
    let last = data.last_mut().ok_or_else(|| invalid("invalid padded BoC cell"))?;
    *last &= !(1 << trailing_zeros);
    let bit_len = data.len() * 8 - (trailing_zeros as usize + 1);
    Ok((data, bit_len))
}

pub(super) fn raw_cell_size(cell: &RawCell, ref_size: usize) -> usize {
    2 + cell.bit_len.div_ceil(8) + cell.references.len() * ref_size
}

pub(super) fn write_raw_cell(output: &mut Vec<u8>, cell: &RawCell, ref_size: usize) {
    let full_bytes = cell.bit_len.is_multiple_of(8);
    let data_len = cell.bit_len.div_ceil(8);
    output.push(cell.references.len() as u8);
    output.push((data_len * 2 - usize::from(!full_bytes)) as u8);

    if !full_bytes && data_len > 0 {
        output.extend_from_slice(&cell.data[..data_len - 1]);
        let padding_bits = cell.bit_len % 8;
        output.push(cell.data[data_len - 1] | (1 << (8 - padding_bits - 1)));
    } else {
        output.extend_from_slice(&cell.data[..data_len]);
    }

    for reference in &cell.references {
        write_var_uint(output, *reference, ref_size);
    }
}

pub(super) fn raw_cell_from_cell(cell: &Cell, indexed_cells: &IndexedCells) -> Result<RawCell, SignerError> {
    let references = cell
        .references()
        .iter()
        .map(|reference| {
            indexed_cells
                .get(&reference.cell_hash())
                .map(|indexed| indexed.borrow().index)
                .ok_or_else(|| invalid("missing referenced cell"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RawCell {
        data: cell.data().to_vec(),
        bit_len: cell.bit_len(),
        references,
    })
}

pub(super) fn write_var_uint(output: &mut Vec<u8>, value: usize, size: usize) {
    for shift in (0..size).rev() {
        output.push(((value >> (shift * 8)) & 0xff) as u8);
    }
}
