use primitives::SignerError;

use super::{invalid, reader::BitReader};

pub(super) const BOC_MAGIC: u32 = 0xb5ee9c72;
const MAX_CELLS: usize = 4096;

// Header flag byte layout: has_idx:1 | has_crc32c:1 | _:1 | _:1 | _:1 | ref_size:3
const REF_SIZE_MASK: u8 = 0b0000_0111;
const HAS_IDX_FLAG: u8 = 0b1000_0000;
const HAS_CRC32C_FLAG: u8 = 0b0100_0000;
const MAX_REF_BYTES: usize = 4;
const MAX_OFF_BYTES: usize = 8;

pub(super) struct BocHeader {
    pub has_idx: bool,
    pub has_crc32c: bool,
    pub ref_bytes: usize,
    pub off_bytes: usize,
    pub cells_count: usize,
    pub roots_count: usize,
    pub total_cells_size: usize,
}

impl BocHeader {
    pub(super) fn parse(reader: &mut BitReader<'_>) -> Result<Self, SignerError> {
        if reader.read_u32()? != BOC_MAGIC {
            return Err(invalid("unsupported BoC magic"));
        }

        let flags = reader.read_u8()?;
        let ref_bytes = (flags & REF_SIZE_MASK) as usize;
        if ref_bytes == 0 || ref_bytes > MAX_REF_BYTES {
            return Err(invalid("unsupported BoC size"));
        }

        let off_bytes = reader.read_u8()? as usize;
        if off_bytes == 0 || off_bytes > MAX_OFF_BYTES {
            return Err(invalid("unsupported BoC offset size"));
        }

        let cells_count = reader.read_var_uint(ref_bytes)?;
        let roots_count = reader.read_var_uint(ref_bytes)?;
        let absent_count = reader.read_var_uint(ref_bytes)?;
        let total_cells_size = reader.read_var_uint(off_bytes)?;

        if cells_count == 0 || cells_count > MAX_CELLS {
            return Err(invalid("unsupported BoC cell count"));
        }
        if roots_count == 0 || roots_count > cells_count {
            return Err(invalid("unsupported BoC root count"));
        }
        if roots_count + absent_count > cells_count {
            return Err(invalid("invalid BoC absent count"));
        }

        Ok(Self {
            has_idx: flags & HAS_IDX_FLAG != 0,
            has_crc32c: flags & HAS_CRC32C_FLAG != 0,
            ref_bytes,
            off_bytes,
            cells_count,
            roots_count,
            total_cells_size,
        })
    }
}
