use super::WhaleMove;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: f64,
    pub flag: TTFlag,
    pub best_move: Option<WhaleMove>,
    pub age: u8,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize,
    pub current_age: u8,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(65_536)
    }
}

impl TranspositionTable {
    pub fn new(capacity: usize) -> Self {
        let size = capacity.max(1024).next_power_of_two();
        Self {
            entries: vec![None; size],
            mask: size - 1,
            current_age: 0,
        }
    }

    pub fn new_search(&mut self) {
        self.current_age = self.current_age.wrapping_add(1);
    }

    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let idx = (key as usize) & self.mask;
        match &self.entries[idx] {
            Some(entry) if entry.key == key => Some(entry),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn store(
        &mut self,
        key: u64,
        depth: u8,
        score: f64,
        flag: TTFlag,
        best_move: Option<WhaleMove>,
    ) {
        let idx = (key as usize) & self.mask;
        let should_replace = match &self.entries[idx] {
            None => true,
            Some(existing) => {
                if existing.key == key {
                    depth >= existing.depth || flag == TTFlag::Exact
                } else {
                    existing.age != self.current_age || depth >= existing.depth
                }
            }
        };

        if should_replace {
            let preserved_move = if best_move.is_some() {
                best_move
            } else if let Some(existing) = &self.entries[idx] {
                if existing.key == key {
                    existing.best_move
                } else {
                    None
                }
            } else {
                None
            };

            self.entries[idx] = Some(TTEntry {
                key,
                depth,
                score,
                flag,
                best_move: preserved_move,
                age: self.current_age,
            });
        }
    }
}
