use std::ops::{BitAnd, BitOr, BitOrAssign};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct OPerms(u32);

#[allow(dead_code)]
impl OPerms {
    pub const READ: Self = Self(0b0001);
    pub const WRITE: Self = Self(0b0010);
    pub const APPEND: Self = Self(0b0100);
    pub const CREATE: Self = Self(0b1000);
    pub const TRUNC: Self = Self(0b1_0000);

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for OPerms {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for OPerms {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for OPerms {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
