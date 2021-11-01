//! Trait for types that implement field arithmetic.

use rand::{CryptoRng, Rng};
use std::hash::Hash;
use std::ops::{AddAssign, Mul, Sub};

/// Trait for types that implement field arithmetic.
pub trait Field: Copy + Eq + Hash + From<u8> + Sub<Output = Self>
where
    for<'a> Self: AddAssign<&'a Self>,
    for<'a> Self: Mul<&'a Self, Output = Self>,
{
    /// The neutral element for addition.
    const ZERO: Self;
    /// The neutral element for multiplication.
    const ONE: Self;

    /// Samples a field element uniformly at random.
    fn uniform<R: Rng + CryptoRng + ?Sized>(rng: &mut R) -> Self;
    /// Inverts an element.
    fn invert(self) -> Self;
    /// Function that computes `Self::from(lhs) - Self::from(rhs)`, allowing implementations to
    /// apply any relevant optimization.
    fn from_diff(lhs: u8, rhs: u8) -> Self;

    /// Parses a field element from a byte slice. Returns `None` if the parsing fails.
    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}
