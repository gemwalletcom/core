mod bag;
mod builder;
mod cell;
mod error;
mod header;
mod indexed_cell;
mod raw_cell;
mod reader;
mod writer;

pub use bag::BagOfCells;
pub use builder::CellBuilder;
pub use cell::{Cell, CellArc};
pub use error::TvmError;
pub use reader::BitReader;
pub use writer::BitWriter;

fn invalid(msg: &'static str) -> TvmError {
    TvmError::new(msg)
}
