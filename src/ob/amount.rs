use std::ops::{Add, AddAssign, Mul, MulAssign, SubAssign};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Amount {
    pub as_int: i64, // "cents"
}

impl Amount {
    pub fn new() -> Self {
        Amount { as_int: 0 }
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::new()
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.as_int += rhs.as_int;
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Amount {
            as_int: self.as_int + rhs.as_int,
        }
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.as_int -= rhs.as_int;
    }
}

impl MulAssign<i64> for Amount {
    fn mul_assign(&mut self, rhs: i64) {
        self.as_int *= rhs;
    }
}

impl Mul<i64> for Amount {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Amount {
            as_int: self.as_int * rhs,
        }
    }
}
