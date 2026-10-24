#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Var(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lit(pub u32);

impl Lit {
    #[inline]
    pub fn new(var: Var, sign: bool) -> Self {
        Lit((var.0 << 1) | (sign as u32))
    }

    #[inline]
    pub fn var(self) -> Var {
        Var(self.0 >> 1)
    }

    #[inline]
    pub fn sign(self) -> bool {
        (self.0 & 1) != 0
    }

    #[inline]
    pub fn negate(self) -> Self {
        Lit(self.0 ^ 1)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}