mod bag;
mod builder;
mod cell;
mod header;
mod indexed_cell;
mod raw_cell;
mod reader;
mod writer;

pub use bag::BagOfCells;
pub use builder::CellBuilder;
pub use cell::{Cell, CellArc};

use primitives::SignerError;

fn invalid(msg: &'static str) -> SignerError {
    SignerError::invalid_input(msg)
}
