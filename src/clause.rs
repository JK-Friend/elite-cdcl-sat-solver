use crate::types::Lit;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClauseId(pub usize);

#[derive(Clone, Debug)]
pub struct Clause {
    pub lits: Vec<Lit>,
    pub is_learned: bool,
}

impl Clause {
    #[inline]
    pub fn new(lits: Vec<Lit>, is_learned: bool) -> Self {
        Clause { lits, is_learned }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lits.is_empty()
    }
}