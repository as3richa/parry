use std::clone::Clone;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

pub(crate) trait Field:
    Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + Div<Output = Self>
    + DivAssign
    + Sized
    + Copy
    + Clone
    + Debug
    + PartialEq
{
    fn zero() -> Self;
    fn one() -> Self;
}
