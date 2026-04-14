use crc::Crc;
use gem_encoding::{EncodingError, decode_base64, decode_base64_no_pad, decode_base64_url, decode_base64_url_padded, encode_base64};
use primitives::SignerError;

use super::cell::{Cell, CellArc};
use super::header::{BOC_MAGIC, BocHeader};
use super::indexed_cell::{build_index, ordered_indexed_cells};
use super::invalid;
use super::raw_cell::{RawCell, raw_cell_from_cell, raw_cell_size, read_raw_cell, write_raw_cell, write_var_uint};
use super::reader::BitReader;

const CRC32C: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

fn decode_boc_base64(value: &str) -> Result<Vec<u8>, EncodingError> {
    decode_base64(value)
        .or_else(|_| decode_base64_no_pad(value))
        .or_else(|_| decode_base64_url_padded(value))
        .or_else(|_| decode_base64_url(value))
}

#[derive(Clone, Debug, Default)]
pub struct BagOfCells {
    roots: Vec<CellArc>,
}

impl BagOfCells {
    pub fn from_root(root: Cell) -> Self {
        Self { roots: vec![root.into_arc()] }
    }

    pub fn parse_base64(value: &str) -> Result<Self, SignerError> {
        let bytes = decode_boc_base64(value).map_err(|_| invalid("invalid base64 BoC"))?;
        Self::parse(&bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, SignerError> {
        let mut reader = BitReader::new(bytes);
        let header = BocHeader::parse(&mut reader)?;

        let root_indexes = (0..header.roots_count).map(|_| reader.read_var_uint(header.ref_bytes)).collect::<Result<Vec<_>, _>>()?;

        if header.has_idx {
            reader.skip(header.cells_count * header.off_bytes)?;
        }

        let raw_cells = read_all_raw_cells(&mut reader, &header)?;

        if header.has_crc32c {
            validate_crc32c(&mut reader, bytes)?;
        }

        if reader.remaining() != 0 {
            return Err(invalid("unexpected trailing BoC bytes"));
        }

        let cells = build_cell_tree(&raw_cells)?;
        let roots = resolve_indexes(&root_indexes, &cells, "BoC root out of bounds")?;
        Ok(Self { roots })
    }

    pub fn single_root(&self) -> Result<&CellArc, SignerError> {
        match self.roots.as_slice() {
            [root] => Ok(root),
            _ => Err(invalid("BoC must contain exactly one root")),
        }
    }

    pub fn to_base64(&self, with_crc32c: bool) -> Result<String, SignerError> {
        Ok(encode_base64(&self.serialize(with_crc32c)?))
    }

    pub fn serialize(&self, with_crc32c: bool) -> Result<Vec<u8>, SignerError> {
        let indexed_cells = build_index(&self.roots);
        let ordered_cells = ordered_indexed_cells(&indexed_cells);

        let raw_cells = ordered_cells
            .iter()
            .map(|indexed| raw_cell_from_cell(&indexed.borrow().cell, &indexed_cells))
            .collect::<Result<Vec<_>, _>>()?;

        let root_indexes = self
            .roots
            .iter()
            .map(|root| indexed_cells.get(&root.cell_hash()).map(|indexed| indexed.borrow().index).ok_or_else(|| invalid("missing BoC root")))
            .collect::<Result<Vec<_>, _>>()?;

        let ref_bytes = bytes_needed(raw_cells.len());
        let total_cells_size = raw_cells.iter().map(|cell| raw_cell_size(cell, ref_bytes)).sum::<usize>();
        let offset_bytes = bytes_needed(total_cells_size.max(1));

        let mut output = Vec::new();
        output.extend_from_slice(&BOC_MAGIC.to_be_bytes());
        output.push(((with_crc32c as u8) << 6) | (ref_bytes as u8));
        output.push(offset_bytes as u8);
        write_var_uint(&mut output, raw_cells.len(), ref_bytes);
        write_var_uint(&mut output, root_indexes.len(), ref_bytes);
        write_var_uint(&mut output, 0, ref_bytes);
        write_var_uint(&mut output, total_cells_size, offset_bytes);
        for root_index in &root_indexes {
            write_var_uint(&mut output, *root_index, ref_bytes);
        }
        for cell in &raw_cells {
            write_raw_cell(&mut output, cell, ref_bytes);
        }

        if with_crc32c {
            output.extend_from_slice(&CRC32C.checksum(&output).to_le_bytes());
        }

        Ok(output)
    }
}

fn read_all_raw_cells(reader: &mut BitReader<'_>, header: &BocHeader) -> Result<Vec<RawCell>, SignerError> {
    let start = reader.position();
    let raw_cells = (0..header.cells_count).map(|_| read_raw_cell(reader, header.ref_bytes)).collect::<Result<Vec<_>, _>>()?;

    if reader.position().saturating_sub(start) != header.total_cells_size {
        return Err(invalid("BoC cell size does not match header"));
    }
    Ok(raw_cells)
}

fn validate_crc32c(reader: &mut BitReader<'_>, bytes: &[u8]) -> Result<(), SignerError> {
    let expected = reader.read_u32_le()?;
    let payload_end = bytes.len().checked_sub(4).ok_or_else(|| invalid("invalid BoC length"))?;
    if expected != CRC32C.checksum(&bytes[..payload_end]) {
        return Err(invalid("invalid BoC crc32c"));
    }
    Ok(())
}

fn build_cell_tree(raw_cells: &[RawCell]) -> Result<Vec<CellArc>, SignerError> {
    let total = raw_cells.len();
    let mut cells = Vec::with_capacity(total);
    for (reverse_index, raw) in raw_cells.iter().enumerate().rev() {
        let references = raw
            .references
            .iter()
            .map(|reference| {
                if *reference <= reverse_index {
                    return Err(invalid("BoC references must point to later cells"));
                }
                resolve_reverse_index(*reference, total, &cells, "BoC reference out of bounds")
            })
            .collect::<Result<Vec<_>, _>>()?;
        cells.push(Cell::new(raw.data.clone(), raw.bit_len, references)?.into_arc());
    }
    Ok(cells)
}

fn resolve_indexes(indexes: &[usize], cells: &[CellArc], error: &'static str) -> Result<Vec<CellArc>, SignerError> {
    indexes.iter().map(|index| resolve_reverse_index(*index, cells.len(), cells, error)).collect()
}

fn resolve_reverse_index(index: usize, total: usize, cells: &[CellArc], error: &'static str) -> Result<CellArc, SignerError> {
    let built_index = total.checked_sub(1 + index).ok_or_else(|| invalid(error))?;
    cells.get(built_index).cloned().ok_or_else(|| invalid(error))
}

fn bytes_needed(value: usize) -> usize {
    let value = value.max(1);
    let bits = usize::BITS as usize - value.leading_zeros() as usize;
    bits.div_ceil(8)
}
