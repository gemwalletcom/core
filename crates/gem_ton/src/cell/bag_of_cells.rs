use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD;

use crate::cell::*;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct BagOfCells {
    pub roots: Vec<ArcCell>,
}

impl BagOfCells {
    pub fn new(roots: &[ArcCell]) -> BagOfCells {
        BagOfCells { roots: roots.to_vec() }
    }

    pub fn from_root(root: Cell) -> BagOfCells {
        let arc = Arc::new(root);
        BagOfCells { roots: vec![arc] }
    }

    pub fn add_root(&mut self, root: Cell) {
        let arc = Arc::new(root);
        self.roots.push(arc)
    }

    pub fn num_roots(&self) -> usize {
        self.roots.len()
    }

    pub fn root(&self, idx: usize) -> Result<&ArcCell, TonCellError> {
        self.roots
            .get(idx)
            .ok_or_else(|| TonCellError::boc_deserialization_error(format!("Invalid root index: {}, BoC contains {} roots", idx, self.roots.len())))
    }

    pub fn single_root(&self) -> Result<&ArcCell, TonCellError> {
        let root_count = self.roots.len();
        if root_count == 1 {
            Ok(&self.roots[0])
        } else {
            Err(TonCellError::CellParserError(format!("Single root expected, got {}", root_count)))
        }
    }

    pub fn parse(serial: &[u8]) -> Result<BagOfCells, TonCellError> {
        let raw = RawBagOfCells::parse(serial)?;
        let num_cells = raw.cells.len();
        let mut cells: Vec<ArcCell> = Vec::new();
        for i in (0..num_cells).rev() {
            let raw_cell = &raw.cells[i];
            let mut cell = Cell {
                data: raw_cell.data.clone(),
                bit_len: raw_cell.bit_len,
                references: Vec::new(),
            };
            for r in &raw_cell.references {
                if *r <= i {
                    return Err(TonCellError::boc_deserialization_error("References to previous cells are not supported"));
                }
                cell.references.push(cells[num_cells - 1 - r].clone());
            }
            cells.push(Arc::new(cell));
        }
        let roots: Vec<ArcCell> = raw.roots.iter().map(|r| cells[num_cells - 1 - r].clone()).collect();
        Ok(BagOfCells { roots })
    }

    pub fn parse_hex(hex: &str) -> Result<BagOfCells, TonCellError> {
        let str: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
        let bin = hex::decode(str.as_str()).map_boc_deserialization_error()?;
        Self::parse(&bin)
    }

    pub fn parse_base64(base64: &str) -> Result<BagOfCells, TonCellError> {
        let bin = STANDARD.decode(base64).map_boc_deserialization_error()?;
        Self::parse(&bin)
    }

    pub fn serialize(&self, has_crc32: bool) -> Result<Vec<u8>, TonCellError> {
        let raw = self.to_raw()?;
        raw.serialize(has_crc32)
    }

    /// Traverses all cells, fills all_cells set and inbound references map.
    fn traverse_cell_tree(cell: &ArcCell, all_cells: &mut HashSet<ArcCell>, in_refs: &mut HashMap<ArcCell, HashSet<ArcCell>>) -> Result<(), TonCellError> {
        if !all_cells.contains(cell) {
            all_cells.insert(cell.clone());
            for r in &cell.references {
                if r == cell {
                    return Err(TonCellError::BagOfCellsDeserializationError("Cell must not reference itself".to_string()));
                }
                let maybe_refs = in_refs.get_mut(&r.clone());
                match maybe_refs {
                    Some(refs) => {
                        refs.insert(cell.clone());
                    }
                    None => {
                        let mut refs: HashSet<ArcCell> = HashSet::new();
                        refs.insert(cell.clone());
                        in_refs.insert(r.clone(), refs);
                    }
                }
                Self::traverse_cell_tree(r, all_cells, in_refs)?;
            }
        }
        Ok(())
    }

    /// Constructs raw representation of BagOfCells
    pub(crate) fn to_raw(&self) -> Result<RawBagOfCells, TonCellError> {
        let mut all_cells: HashSet<ArcCell> = HashSet::new();
        let mut in_refs: HashMap<ArcCell, HashSet<ArcCell>> = HashMap::new();
        for r in &self.roots {
            Self::traverse_cell_tree(r, &mut all_cells, &mut in_refs)?;
        }
        let mut no_in_refs: HashSet<ArcCell> = HashSet::new();
        for c in &all_cells {
            if !in_refs.contains_key(c) {
                no_in_refs.insert(c.clone());
            }
        }
        let mut ordered_cells: Vec<ArcCell> = Vec::new();
        let mut indices: HashMap<ArcCell, usize> = HashMap::new();
        while !no_in_refs.is_empty() {
            let cell = no_in_refs.iter().next().unwrap().clone();
            ordered_cells.push(cell.clone());
            indices.insert(cell.clone(), indices.len());
            for child in &cell.references {
                if let Some(refs) = in_refs.get_mut(child) {
                    refs.remove(&cell);
                    if refs.is_empty() {
                        no_in_refs.insert(child.clone());
                        in_refs.remove(child);
                    }
                }
            }
            no_in_refs.remove(&cell);
        }
        if !in_refs.is_empty() {
            return Err(TonCellError::CellBuilderError(
                "Can't construct topological ordering: cycle detected".to_string(),
            ));
        }
        let mut cells: Vec<RawCell> = Vec::new();
        for cell in &ordered_cells {
            let refs: Vec<usize> = cell.references.iter().map(|c| *indices.get(c).unwrap()).collect();
            let raw = RawCell {
                data: cell.data.clone(),
                bit_len: cell.bit_len,
                references: refs,
                max_level: cell.get_max_level(),
            };
            cells.push(raw);
        }
        let roots: Vec<usize> = self.roots.iter().map(|c| *indices.get(c).unwrap()).collect();
        Ok(RawBagOfCells { cells, roots })
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::{BagOfCells, CellBuilder};

    #[test]
    fn it_constructs_raw() -> anyhow::Result<()> {
        let leaf = CellBuilder::new().store_byte(10)?.build()?;
        let inter = CellBuilder::new().store_byte(20)?.store_child(leaf)?.build()?;
        let root = CellBuilder::new().store_byte(30)?.store_child(inter)?.build()?;
        let boc = BagOfCells::from_root(root);
        let _raw = boc.to_raw()?;
        Ok(())
    }
}
