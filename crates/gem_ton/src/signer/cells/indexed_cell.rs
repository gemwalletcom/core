use std::cell::RefCell;
use std::collections::BTreeMap;

use super::cell::{CellArc, MAX_CELL_REFERENCES};

pub(super) type IndexedCellRef = RefCell<IndexedCell>;
pub(super) type IndexedCells = BTreeMap<[u8; 32], IndexedCellRef>;

#[derive(Clone)]
pub(super) struct IndexedCell {
    pub index: usize,
    pub cell: CellArc,
}

pub(super) fn build_index(roots: &[CellArc]) -> IndexedCells {
    let mut indexed_cells = BTreeMap::new();
    let mut next_index = 0usize;

    let mut frontier = roots.to_vec();
    while !frontier.is_empty() {
        let mut next_frontier = Vec::with_capacity(frontier.len() * MAX_CELL_REFERENCES);
        for cell in frontier {
            if indexed_cells.contains_key(&cell.hash) {
                continue;
            }
            indexed_cells.insert(
                cell.hash,
                RefCell::new(IndexedCell {
                    index: next_index,
                    cell: cell.clone(),
                }),
            );
            next_index += 1;
            next_frontier.extend(cell.references.iter().cloned());
        }
        frontier = next_frontier;
    }

    loop {
        let mut reordered = false;
        for parent_hash in indexed_cells.keys().copied().collect::<Vec<_>>() {
            let Some(parent) = indexed_cells.get(&parent_hash) else {
                continue;
            };
            let parent_index = parent.borrow().index;
            let reference_hashes = parent.borrow().cell.references.iter().map(|reference| reference.hash).collect::<Vec<_>>();
            for reference_hash in reference_hashes {
                let Some(reference) = indexed_cells.get(&reference_hash) else {
                    continue;
                };
                if reference.borrow().index < parent_index {
                    reference.borrow_mut().index = next_index;
                    next_index += 1;
                    reordered = true;
                }
            }
        }
        if !reordered {
            break;
        }
    }

    indexed_cells
}

pub(super) fn ordered_indexed_cells(indexed_cells: &IndexedCells) -> Vec<&IndexedCellRef> {
    let mut ordered = indexed_cells.values().collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|cell| cell.borrow().index);
    for (real_index, cell) in ordered.iter().enumerate() {
        cell.borrow_mut().index = real_index;
    }
    ordered
}
