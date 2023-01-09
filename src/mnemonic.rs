use std::fmt;
use std::ops::*;
use std::marker::PhantomData;
use rand::{CryptoRng, Rng};

use horcrux::field::Field;
use horcrux::gf2n::{GF128, GF256};
use horcrux::shamir::{Shamir};

#[cfg(feature = "bip39")]
use bip39::{Mnemonic, Language};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bip39<F> {
    base: F
}

impl<F> Bip39<F> {
    pub const fn new(base: F) -> Self {
        Self { base }
    }
}

#[cfg(feature = "bip39")]
fn mnemonic<'a, I>(bs: I) -> Mnemonic where I: Iterator<Item = &'a u64> {
    let bytes: Vec<u8> = bs.map(|w| u64::to_be_bytes(*w)).flatten().collect();
    Mnemonic::from_entropy(&bytes, Language::English).unwrap()
}

#[cfg(not(feature = "bip39"))]
fn mnemonic<I>(_: I) -> ! {
    panic!("bip39 mnemonics requires the bip39 feature flag")
}

impl fmt::Display for Bip39<GF128> {
    #[allow(unreachable_code)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", mnemonic(self.base.words().iter()))
    }
}

impl fmt::Display for Bip39<GF256> {
    #[allow(unreachable_code)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", mnemonic(self.base.words().iter()))
    }
}

impl<F: Field> Field for Bip39<F> {
    const ZERO: Self = Self::new(F::ZERO);
    const ONE: Self = Self::new(F::ONE);

    fn uniform<R: Rng + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Self::new(F::uniform(rng))
    }

    fn invert(self) -> Self {
        Self::new(self.base.invert())
    }

    fn from_diff(lhs: u8, rhs: u8) -> Self {
        Self::new(F::from_diff(lhs, rhs))
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        F::from_bytes(bytes).map(Self::new)
    }
}

impl<F: Field> From<u8> for Bip39<F> {
    fn from(word: u8) -> Self {
        Self::new(F::from(word))
    }
}

impl<F: Field> AddAssign<&Self> for Bip39<F> {
    fn add_assign(&mut self, other: &Self) {
        self.base += &other.base
    }
}

impl<F: Field> Sub for Bip39<F> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.base - other.base)
    }
}

impl<F: Field> Mul<&Self> for Bip39<F> {
    type Output = Self;
    fn mul(self, other: &Self) -> Self {
        Self::new(self.base * &other.base)
    }
}

pub struct Bip39Shamir<S> {
    _phantom_s: PhantomData<S>
}

impl<F: Field, S: Shamir<F>> Shamir<F> for Bip39Shamir<S> {
    type X = S::X;
    type Share = S::Share;

    fn share(x: Self::X, y: F) -> Self::Share {
        S::share(x, y)
    }

    fn split(secret: &F, k: usize, n: usize) -> Vec<Self::Share> {
        S::split(secret, k, n)
    }

    fn reconstruct(shares: &[Self::Share], k: usize) -> Option<F> {
        S::reconstruct(shares, k)
    }

    fn reconstruct_at(shares: &[Self::Share], k: usize, x: Self::X) -> Option<Self::Share> {
        S::reconstruct_at(shares, k, x)
    }

    fn parse_x(s: &str) -> Option<Self::X> {
        S::parse_x(s)
    }

    #[cfg(feature = "bip39")]
    fn parse_share(s: &str) -> Option<Self::Share> {
        use regex::Regex;
        let regex = Regex::new(r"^([0-9]+)\|(.+)$").unwrap();
        let captures = regex.captures(s)?;
        let x = Self::parse_x(&captures[1])?;
        let mnemonic = Mnemonic::from_phrase(&captures[2], Language::English).ok()?;
        let y = F::from_bytes(mnemonic.entropy())?;
        Some(S::share(x, y))
    }

    #[cfg(not(feature = "bip39"))]
    fn parse_share(_: &str) -> Option<Self::Share> {
        panic!("bip39 mnemonics requires the bip39 feature flag")
    }
}
