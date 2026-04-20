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
    pub(super) fn try_parse(reader: &mut BitReader<'_>) -> Option<Self> {
        if reader.read_u32().ok()? != BOC_MAGIC {
            return None;
        }

        let flags = reader.read_u8().ok()?;
        let ref_bytes = (flags & REF_SIZE_MASK) as usize;
        if ref_bytes == 0 || ref_bytes > MAX_REF_BYTES {
            return None;
        }

        let off_bytes = reader.read_u8().ok()? as usize;
        if off_bytes == 0 || off_bytes > MAX_OFF_BYTES {
            return None;
        }

        let cells_count = reader.read_var_uint(ref_bytes).ok()?;
        let roots_count = reader.read_var_uint(ref_bytes).ok()?;
        let absent_count = reader.read_var_uint(ref_bytes).ok()?;
        let total_cells_size = reader.read_var_uint(off_bytes).ok()?;

        if cells_count == 0 || cells_count > MAX_CELLS {
            return None;
        }
        if roots_count == 0 || roots_count > cells_count {
            return None;
        }
        if roots_count + absent_count > cells_count {
            return None;
        }

        Some(Self {
            has_idx: flags & HAS_IDX_FLAG != 0,
            has_crc32c: flags & HAS_CRC32C_FLAG != 0,
            ref_bytes,
            off_bytes,
            cells_count,
            roots_count,
            total_cells_size,
        })
    }

    pub(super) fn parse(reader: &mut BitReader<'_>) -> Result<Self, SignerError> {
        Self::try_parse(reader).ok_or_else(|| invalid("invalid BoC header"))
    }
}
