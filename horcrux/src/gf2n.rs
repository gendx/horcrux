//! Generic implementation of a finite field GF(2^n).
//!
//! This is based on the existence of a irreducible polynomial of the form
//! `x^n + x^a + x^b + x^c + 1`, where `0 < c < b < a < n`.

use crate::field::Field;
use rand::distributions::{Distribution, Standard};
use rand::{CryptoRng, Rng};
#[cfg(feature = "parse")]
use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, BitAnd, BitXor, BitXorAssign, Mul, MulAssign, Not, Shl, Shr, Sub};

/// Trait for words that can be used for the representation of elements of GF(2^n).
pub trait Word:
    Copy
    + Eq
    + Hash
    + Debug
    + From<u8>
    + BitAnd<Output = Self>
    + BitXorAssign
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
{
    /// Zero.
    const ZERO: Self;
    /// One.
    const ONE: Self;
    /// Number of bytes in the size of the type.
    const NBYTES: usize = std::mem::size_of::<Self>();
    /// Number of bits in the size of the type.
    const NBITS: usize = 8 * Self::NBYTES;
    /// Base-2 logarithm of `NBITS`.
    #[cfg(test)]
    const MASK_BITS: usize;
    /// Mask with `MASK_BITS` ones at the end.
    #[cfg(test)]
    const MASK: usize = !(!1 << (Self::MASK_BITS - 1));

    /// Parses a word from a byte slice. Panics if the slice length is not `NBYTES`.
    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self;
}

// TODO: Make this implementation generic once const generics allow it.
impl Word for u128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[cfg(test)]
    const MASK_BITS: usize = 7;

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self {
        let array = bytes.try_into().unwrap();
        u128::from_be_bytes(array)
    }
}

impl Word for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[cfg(test)]
    const MASK_BITS: usize = 6;

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self {
        let array = bytes.try_into().unwrap();
        u64::from_be_bytes(array)
    }
}

impl Word for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[cfg(test)]
    const MASK_BITS: usize = 5;

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self {
        let array = bytes.try_into().unwrap();
        u32::from_be_bytes(array)
    }
}

impl Word for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[cfg(test)]
    const MASK_BITS: usize = 4;

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self {
        let array = bytes.try_into().unwrap();
        u16::from_be_bytes(array)
    }
}

impl Word for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[cfg(test)]
    const MASK_BITS: usize = 3;

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Self {
        let array = bytes.try_into().unwrap();
        u8::from_be_bytes(array)
    }
}

/// Implementation of a binary field GF(2^n), with `W::NBYTES * NWORDS` bits, using the
/// irreducible polynomial `x^n + x^a + x^b + x^c + 1`.
#[derive(Clone, Copy)]
pub struct GF2n<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> {
    words: [W; NWORDS],
}

/// Finite field GF(2^8) implemented with 8-bit words and using the following irreducible
/// polynomial: `x^8 + x^4 + x^3 + x + 1`.
pub type GF8 = GF2n<u8, 1, 4, 3, 1>;
/// Finite field GF(2^16) implemented with 16-bit words and using the following irreducible
/// polynomial: `x^16 + x^5 + x^3 + x + 1`.
pub type GF16 = GF2n<u16, 1, 5, 3, 1>;
/// Finite field GF(2^32) implemented with 32-bit words and using the following irreducible
/// polynomial: `x^32 + x^7 + x^3 + x^2 + 1`.
pub type GF32 = GF2n<u32, 1, 7, 3, 2>;
/// Finite field GF(2^64) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^64 + x^4 + x^3 + x + 1`.
pub type GF64 = GF2n<u64, 1, 4, 3, 1>;
/// Finite field GF(2^64) implemented with 32-bit words and using the following irreducible
/// polynomial: `x^64 + x^4 + x^3 + x + 1`.
#[cfg(test)]
pub type GF64u32 = GF2n<u32, 2, 4, 3, 1>;
/// Finite field GF(2^128) implemented with 128-bit words and using the following irreducible
/// polynomial: `x^128 + x^7 + x^2 + x + 1`.
#[cfg(test)]
pub type GF128u128 = GF2n<u128, 1, 7, 2, 1>;
/// Finite field GF(2^128) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^128 + x^7 + x^2 + x + 1`.
pub type GF128 = GF2n<u64, 2, 7, 2, 1>;
/// Finite field GF(2^128) implemented with 32-bit words and using the following irreducible
/// polynomial: `x^128 + x^7 + x^2 + x + 1`.
#[cfg(test)]
pub type GF128u32 = GF2n<u32, 4, 7, 2, 1>;
/// Finite field GF(2^256) implemented with 128-bit words and using the following irreducible
/// polynomial: `x^256 + x^10 + x^5 + x^2 + 1`.
#[cfg(test)]
pub type GF256u128 = GF2n<u128, 2, 10, 5, 2>;
/// Finite field GF(2^256) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^256 + x^10 + x^5 + x^2 + 1`.
pub type GF256 = GF2n<u64, 4, 10, 5, 2>;
/// Finite field GF(2^256) implemented with 32-bit words and using the following irreducible
/// polynomial: `x^256 + x^10 + x^5 + x^2 + 1`.
#[cfg(test)]
pub type GF256u32 = GF2n<u32, 8, 10, 5, 2>;

/// Finite field GF(2^192) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^192 + x^7 + x^2 + x + 1`.
pub type GF192 = GF2n<u64, 3, 7, 2, 1>;
/// Finite field GF(2^384) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^384 + x^12 + x^3 + x^2 + 1`.
pub type GF384 = GF2n<u64, 6, 12, 3, 2>;
/// Finite field GF(2^512) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^512 + x^8 + x^5 + x^2 + 1`.
pub type GF512 = GF2n<u64, 8, 8, 5, 2>;
/// Finite field GF(2^768) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^768 + x^19 + x^17 + x^4 + 1`.
pub type GF768 = GF2n<u64, 12, 19, 17, 4>;
/// Finite field GF(2^1024) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^1024 + x^19 + x^6 + x + 1`.
pub type GF1024 = GF2n<u64, 16, 19, 6, 1>;
/// Finite field GF(2^1536) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^1536 + x^21 + x^6 + x^2 + 1`.
pub type GF1536 = GF2n<u64, 24, 21, 6, 2>;
/// Finite field GF(2^2048) implemented with 64-bit words and using the following irreducible
/// polynomial: `x^2048 + x^19 + x^14 + x^13 + 1`.
pub type GF2048 = GF2n<u64, 32, 19, 14, 13>;

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Debug
    for GF2n<W, NWORDS, A, B, C>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match W::NBITS {
            8 => f.write_fmt(format_args!("{:02x?}", &self.words as &[W])),
            16 => f.write_fmt(format_args!("{:04x?}", &self.words as &[W])),
            32 => f.write_fmt(format_args!("{:08x?}", &self.words as &[W])),
            64 => f.write_fmt(format_args!("{:016x?}", &self.words as &[W])),
            128 => f.write_fmt(format_args!("{:032x?}", &self.words as &[W])),
            _ => f.write_fmt(format_args!("{:x?}", &self.words as &[W])),
        }
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Display
    for GF2n<W, NWORDS, A, B, C>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for d in &self.words as &[W] {
            match W::NBITS {
                8 => f.write_fmt(format_args!("{d:02x?}"))?,
                16 => f.write_fmt(format_args!("{d:04x?}"))?,
                32 => f.write_fmt(format_args!("{d:08x?}"))?,
                64 => f.write_fmt(format_args!("{d:016x?}"))?,
                128 => f.write_fmt(format_args!("{d:032x?}"))?,
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> PartialEq
    for GF2n<W, NWORDS, A, B, C>
{
    fn eq(&self, other: &Self) -> bool {
        &self.words as &[W] == &other.words as &[W]
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Eq
    for GF2n<W, NWORDS, A, B, C>
{
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Hash
    for GF2n<W, NWORDS, A, B, C>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.words as &[W]).hash(state)
    }
}

trait FieldExt {
    type W;
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> FieldExt
    for GF2n<W, NWORDS, A, B, C>
{
    type W = W;
}

#[cfg(all(
    feature = "clmul",
    target_arch = "x86_64",
    target_feature = "sse2",
    target_feature = "pclmulqdq"
))]
fn mul_clmul_u64<const NWORDS: usize, const A: usize, const B: usize, const C: usize>(
    x: &GF2n<u64, NWORDS, A, B, C>,
    y: &GF2n<u64, NWORDS, A, B, C>,
) -> GF2n<u64, NWORDS, A, B, C> {
    use core::arch::x86_64::{__m128i, _mm_clmulepi64_si128, _mm_set_epi64x, _mm_storeu_si128};

    // Note: we cannot create an array of `NWORDS * 2` elements:
    // error: constant expression depends on a generic parameter
    let mut words = [0u64; NWORDS];
    let mut carry = [0u64; NWORDS];

    for i in 0..NWORDS {
        // Safety: target_feature "sse2" is available in this function.
        let xi: __m128i = unsafe { _mm_set_epi64x(0, x.words[i] as i64) };
        for j in 0..NWORDS {
            // Safety: target_feature "sse2" is available in this function.
            let yj: __m128i = unsafe { _mm_set_epi64x(0, y.words[j] as i64) };
            // Safety: target_feature "pclmulqdq" is available in this function.
            let clmul: __m128i = unsafe { _mm_clmulepi64_si128(xi, yj, 0) };
            let mut cc: [u64; 2] = [0u64, 0u64];
            // Safety:
            // - target_feature "sse2" is available in this function,
            // - cc points to 128 bits (no alignment required by this function).
            unsafe { _mm_storeu_si128(&mut cc as *mut _ as *mut __m128i, clmul) };

            let ij = i + j;
            if ij < NWORDS {
                words[ij] ^= cc[0];
            } else {
                carry[ij - NWORDS] ^= cc[0];
            }

            let ij1 = ij + 1;
            if ij1 < NWORDS {
                words[ij1] ^= cc[1];
            } else {
                carry[ij1 - NWORDS] ^= cc[1];
            }
        }
    }

    GF2n::<u64, NWORDS, A, B, C>::propagate_carries(words, carry)
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize>
    GF2n<W, NWORDS, A, B, C>
{
    #[cfg(test)]
    const NWORDS: usize = NWORDS;
    const NBITS: usize = W::NBITS * NWORDS;
    #[cfg(feature = "parse")]
    const NBYTES: usize = W::NBYTES * NWORDS;

    #[cfg(test)]
    pub const fn new(words: [W; NWORDS]) -> Self {
        Self { words }
    }

    const fn new_small(word: W) -> Self {
        let mut words = [W::ZERO; NWORDS];
        words[0] = word;
        Self { words }
    }

    #[cfg(test)]
    fn get_nonzero_test_values() -> Vec<Self> {
        let all_ones = [!W::ZERO; NWORDS];
        let all_zeros = [W::ZERO; NWORDS];

        let mut values = Vec::new();
        values.push(Self::new(all_ones));

        for i in 0..W::NBITS {
            let word = W::ONE << i;
            for j in 0..NWORDS {
                let mut words = all_zeros;
                words[j] ^= word;
                values.push(Self::new(words));
                let mut words = all_ones;
                words[j] ^= word;
                values.push(Self::new(words));
            }
        }

        values
    }

    #[cfg(test)]
    fn get_test_values() -> Vec<Self> {
        let mut values = Self::get_nonzero_test_values();
        values.push(Self::new([W::ZERO; NWORDS]));
        values
    }

    #[cfg(test)]
    fn xn(n: usize) -> Self {
        debug_assert!(n < Self::NBITS);
        let mut words = [W::ZERO; NWORDS];
        words[n >> W::MASK_BITS] = W::ONE << (n & W::MASK);
        Self { words }
    }

    #[cfg(test)]
    fn get_bit(&self, bit: usize) -> bool {
        debug_assert!(bit < Self::NBITS);
        (self.words[bit >> W::MASK_BITS] >> (bit & W::MASK)) & W::ONE != W::ZERO
    }

    #[cfg(test)]
    fn shl1_ret(mut self) -> Self {
        self.shl1();
        self
    }

    #[cfg(test)]
    fn shl_word_ret(mut self, word: usize) -> Self {
        self.shl_word(word);
        self
    }

    #[cfg(test)]
    fn shlt_ret(mut self) -> Self {
        self.shlt();
        self
    }

    fn shl1(&mut self) {
        let mut carry = W::ZERO;
        for i in 0..NWORDS {
            let d = self.words[i];
            self.words[i] = (d << 1) ^ carry;
            carry = d >> (W::NBITS - 1);
        }
        if carry != W::ZERO {
            self.words[0] ^= W::ONE ^ (W::ONE << A) ^ (W::ONE << B) ^ (W::ONE << C);
        }
    }

    #[cfg(test)]
    fn shl_word(&mut self, shift: usize) {
        debug_assert!(shift != 0 && shift < W::NBITS);
        if NWORDS == 1 {
            let d = self.words[0];
            self.words[0] = d << shift;
            let mut carry = d >> (W::NBITS - shift);
            while carry != W::ZERO {
                self.words[0] ^= carry ^ (carry << A) ^ (carry << B) ^ (carry << C);
                carry = (carry >> (W::NBITS - A))
                    ^ (carry >> (W::NBITS - B))
                    ^ (carry >> (W::NBITS - C));
            }
        } else {
            let mut carry = W::ZERO;
            for i in 0..NWORDS {
                let d = self.words[i];
                self.words[i] = (d << shift) ^ carry;
                carry = d >> (W::NBITS - shift);
            }
            self.words[0] ^= carry ^ (carry << A) ^ (carry << B) ^ (carry << C);
            self.words[1] ^=
                (carry >> (W::NBITS - A)) ^ (carry >> (W::NBITS - B)) ^ (carry >> (W::NBITS - C));
        }
    }

    #[cfg(test)]
    fn shlt(&mut self) {
        if NWORDS == 1 {
            let mut carry = self.words[0];
            self.words[0] = W::ZERO;
            while carry != W::ZERO {
                self.words[0] ^= carry ^ (carry << A) ^ (carry << B) ^ (carry << C);
                carry = (carry >> (W::NBITS - A))
                    ^ (carry >> (W::NBITS - B))
                    ^ (carry >> (W::NBITS - C));
            }
        } else {
            let carry = self.words[NWORDS - 1];
            for i in (1..NWORDS).rev() {
                self.words[i] = self.words[i - 1];
            }
            self.words[0] = carry ^ (carry << A) ^ (carry << B) ^ (carry << C);
            self.words[1] ^=
                (carry >> (W::NBITS - A)) ^ (carry >> (W::NBITS - B)) ^ (carry >> (W::NBITS - C));
        }
    }

    fn mul_as_add(mut self, other: &Self) -> Self {
        let mut result = Self {
            words: [W::ZERO; NWORDS],
        };
        for &word in &other.words as &[W] {
            for i in 0..W::NBITS {
                if word & (W::ONE << i) != W::ZERO {
                    result += &self;
                }
                self.shl1();
            }
        }
        result
    }

    #[cfg(test)]
    fn mul_fused_carry(&self, other: &Self) -> Self {
        // Note: we cannot create an array of `NWORDS * 2` elements:
        // error: constant expression depends on a generic parameter
        let mut words = [W::ZERO; NWORDS];
        let mut carry = [W::ZERO; NWORDS];
        for i in 0..NWORDS {
            let word = other.words[i];
            for j in 0..W::NBITS {
                if word & (W::ONE << j) != W::ZERO {
                    for k in 0..NWORDS {
                        let d = self.words[k] << j;
                        let ki = k + i;
                        if ki < NWORDS {
                            words[ki] ^= d;
                        } else {
                            carry[ki - NWORDS] ^= d;
                        }

                        if j != 0 {
                            let c = self.words[k] >> (W::NBITS - j);
                            let kic = ki + 1;
                            if kic < NWORDS {
                                words[kic] ^= c;
                            } else {
                                carry[kic - NWORDS] ^= c;
                            }
                        }
                    }
                }
            }
        }

        Self::propagate_carries(words, carry)
    }

    #[cfg(any(
        test,
        all(
            feature = "clmul",
            target_arch = "x86_64",
            target_feature = "sse2",
            target_feature = "pclmulqdq"
        )
    ))]
    fn propagate_carries(mut words: [W; NWORDS], carry: [W; NWORDS]) -> Self {
        if NWORDS == 1 {
            let mut c = carry[0];
            while c != W::ZERO {
                words[0] ^= c ^ (c << A) ^ (c << B) ^ (c << C);
                c = (c >> (W::NBITS - A)) ^ (c >> (W::NBITS - B)) ^ (c >> (W::NBITS - C));
            }
        } else {
            for i in 0..NWORDS {
                let c = carry[i];
                words[i] ^= c ^ (c << A) ^ (c << B) ^ (c << C);
                if i + 1 < NWORDS {
                    words[i + 1] ^=
                        (c >> (W::NBITS - A)) ^ (c >> (W::NBITS - B)) ^ (c >> (W::NBITS - C));
                } else {
                    let c = (c >> (W::NBITS - A)) ^ (c >> (W::NBITS - B)) ^ (c >> (W::NBITS - C));
                    words[0] ^= c ^ (c << A) ^ (c << B) ^ (c << C);
                    words[1] ^=
                        (c >> (W::NBITS - A)) ^ (c >> (W::NBITS - B)) ^ (c >> (W::NBITS - C));
                }
            }
        }

        Self { words }
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Field
    for GF2n<W, NWORDS, A, B, C>
where
    Standard: Distribution<W>,
{
    const ZERO: Self = Self::new_small(W::ZERO);
    const ONE: Self = Self::new_small(W::ONE);

    fn uniform<R: Rng + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let mut words = [W::ZERO; NWORDS];
        for word in &mut words as &mut [W] {
            *word = rng.gen();
        }
        Self { words }
    }

    fn invert(mut self) -> Self {
        // Compute x^(2^n - 2)
        let mut result = Self::ONE;
        for _ in 1..Self::NBITS {
            self = self * &self;
            result *= &self;
        }
        result
    }

    fn from_diff(lhs: u8, rhs: u8) -> Self {
        Self::from(lhs ^ rhs)
    }

    #[cfg(feature = "parse")]
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::NBYTES {
            return None;
        }

        let mut words = [W::ZERO; NWORDS];
        for (i, word) in words.iter_mut().enumerate() {
            *word = W::from_bytes(&bytes[i * W::NBYTES..(i + 1) * W::NBYTES]);
        }
        Some(Self { words })
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> From<u8>
    for GF2n<W, NWORDS, A, B, C>
{
    fn from(word: u8) -> Self {
        let mut words = [W::ZERO; NWORDS];
        words[0] = W::from(word);
        Self { words }
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Add
    for GF2n<W, NWORDS, A, B, C>
{
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    #[allow(clippy::needless_range_loop)]
    fn add(self, other: Self) -> Self {
        let mut words = [W::ZERO; NWORDS];
        for i in 0..NWORDS {
            words[i] = self.words[i] ^ other.words[i];
        }
        Self { words }
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> AddAssign<&Self>
    for GF2n<W, NWORDS, A, B, C>
{
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: &Self) {
        for i in 0..NWORDS {
            self.words[i] ^= other.words[i];
        }
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Sub
    for GF2n<W, NWORDS, A, B, C>
{
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, other: Self) -> Self {
        self + other
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Mul<&Self>
    for GF2n<W, NWORDS, A, B, C>
{
    type Output = Self;

    fn mul(self, other: &Self) -> Self {
        #[cfg(all(
            feature = "clmul",
            target_arch = "x86_64",
            target_feature = "sse2",
            target_feature = "pclmulqdq"
        ))]
        if W::NBITS == 64 {
            // Safety: W == u64 when NBITS == 64.
            let x: &GF2n<u64, NWORDS, A, B, C> = unsafe { std::mem::transmute(&self) };
            // Safety: W == u64 when NBITS == 64.
            let y: &GF2n<u64, NWORDS, A, B, C> = unsafe { std::mem::transmute(other) };
            let tmp: GF2n<u64, NWORDS, A, B, C> = mul_clmul_u64(x, y);
            // Safety: W == u64 when NBITS == 64.
            let result: &Self = unsafe { std::mem::transmute(&tmp) };
            return *result;
        }
        self.mul_as_add(other)
    }
}

impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> MulAssign<&Self>
    for GF2n<W, NWORDS, A, B, C>
{
    fn mul_assign(&mut self, other: &Self) {
        *self = *self * other;
    }
}

#[cfg(test)]
impl<W: Word, const NWORDS: usize, const A: usize, const B: usize, const C: usize> Shl<usize>
    for GF2n<W, NWORDS, A, B, C>
{
    type Output = Self;

    fn shl(mut self, mut shift: usize) -> Self {
        debug_assert!(shift < Self::NBITS);
        while shift >= W::NBITS {
            self.shlt();
            shift -= W::NBITS;
        }
        if shift != 0 {
            self.shl_word(shift)
        }
        self
    }
}

#[cfg(test)]
mod test {
    macro_rules! for_field {
        ( $mod:ident, $field:ident, $($tests:tt)* ) => {
            mod $mod {
                type F = super::super::$field;
                $($tests)*
            }
        }
    }

    macro_rules! for_all {
        ( $($tests:tt)* ) => {
            for_field!(gf008, GF8, $($tests)*);
            for_field!(gf016, GF16, $($tests)*);
            for_field!(gf032, GF32, $($tests)*);
            for_field!(gf064, GF64, $($tests)*);
            for_field!(gf064u32, GF64u32, $($tests)*);
            for_field!(gf128, GF128, $($tests)*);
            for_field!(gf128u32, GF128u32, $($tests)*);
            for_field!(gf128u128, GF128u128, $($tests)*);
            for_field!(gf256, GF256, $($tests)*);
            for_field!(gf256u32, GF256u32, $($tests)*);
            for_field!(gf256u128, GF256u128, $($tests)*);
            for_field!(gf512, GF512, $($tests)*);
            for_field!(gf1024, GF1024, $($tests)*);
            for_field!(gf2048, GF2048, $($tests)*);
        };
    }

    macro_rules! for_all_fast {
        ( $($tests:tt)* ) => {
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf008, GF8, $($tests)*);
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf016, GF16, $($tests)*);
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf032, GF32, $($tests)*);
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf064, GF64, $($tests)*);
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf128, GF128, $($tests)*);
            #[cfg(not(debug_assertions))]
            for_field!(fast_gf256, GF256, $($tests)*);
        };
    }

    macro_rules! for_all_clmul {
        ( $($tests:tt)* ) => {
            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf064, GF64, $($tests)*);

            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf128, GF128, $($tests)*);

            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf256, GF256, $($tests)*);

            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf512, GF512, $($tests)*);

            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf1024, GF1024, $($tests)*);

            #[cfg(all(
                feature = "clmul",
                target_arch = "x86_64",
                target_feature = "sse2",
                target_feature = "pclmulqdq"
            ))]
            for_field!(clmul_gf2048, GF2048, $($tests)*);
        };
    }

    for_all! {
        use crate::field::Field;
        use super::super::Word;
        type W = <F as super::super::FieldExt>::W;

        #[test]
        fn get_bit() {
            for i in 0..F::NBITS {
                let xi = F::xn(i);
                for j in 0..F::NBITS {
                    assert_eq!(xi.get_bit(j), i == j);
                }
            }
        }

        #[test]
        fn from_diff() {
            for i in 0..=255 {
                for j in 0..=255 {
                    assert_eq!(F::from_diff(i, j), F::from(i) - F::from(j));
                }
            }
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn add_is_associative() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    for &z in &values {
                        assert_eq!((x + y) + z, x + (y + z));
                    }
                }
            }
        }

        #[test]
        fn add_is_commutative() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    assert_eq!(x + y, y + x);
                }
            }
        }

        #[test]
        fn add_by_zero() {
            let values = F::get_test_values();
            for &x in &values {
                assert_eq!(x + F::ZERO, x);
                assert_eq!(F::ZERO + x, x);
            }
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn mul_is_commutative() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    assert_eq!(x * &y, y * &x);
                }
            }
        }

        #[test]
        fn mul_by_zero() {
            let values = F::get_test_values();
            for &x in &values {
                assert_eq!(x * &F::ZERO, F::ZERO);
                assert_eq!(F::ZERO * &x, F::ZERO);
            }
        }

        #[test]
        fn mul_by_one_left() {
            let values = F::get_test_values();
            for &x in &values {
                assert_eq!(F::ONE * &x, x);
            }
        }

        #[test]
        fn mul_by_one_right() {
            let values = F::get_test_values();
            for &x in &values {
                assert_eq!(x * &F::ONE, x);
            }
        }

        #[test]
        fn mul_by_xn_left() {
            let values = F::get_test_values();
            for i in 0..F::NBITS {
                let xi = F::xn(i);
                for &x in &values {
                    assert_eq!(xi * &x, x << i);
                }
            }
        }

        #[test]
        fn mul_by_xn_right() {
            let values = F::get_test_values();
            for i in 0..F::NBITS {
                let xi = F::xn(i);
                for &x in &values {
                    assert_eq!(x * &xi, x << i);
                }
            }
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn mul_self_invert() {
            let values = F::get_nonzero_test_values();
            for &x in &values {
                let inv = x.invert();
                assert_eq!(x * &inv, F::ONE);
                assert_eq!(inv * &x, F::ONE);
            }
        }

        #[test]
        fn mul_as_add_is_mul_fused_carry() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    assert_eq!(x.mul_as_add(&y), x.mul_fused_carry(&y));
                }
            }
        }

        #[test]
        fn shl1_is_shl_word_1() {
            let values = F::get_test_values();
            for &x in &values {
                assert_eq!(x.shl1_ret(), x.shl_word_ret(1));
            }
        }

        #[test]
        fn shl_word_from_shl1() {
            for x in F::get_test_values() {
                let mut y = x;
                for shift in 1..W::NBITS {
                    y.shl1();
                    assert_eq!(y, x.shl_word_ret(shift));
                }
            }
        }

        #[test]
        fn shlt_from_shl1() {
            let values = F::get_test_values();
            for &x in &values {
                let mut y = x;
                for _ in 0..W::NBITS {
                    y.shl1();
                }
                assert_eq!(y, x.shlt_ret());
            }
        }

        #[test]
        fn shl_from_shl1() {
            let values = F::get_test_values();
            for &x in &values {
                let mut y = x;
                for shift in 0..F::NBITS {
                    assert_eq!(y, x << shift);
                    y.shl1();
                }
            }
        }

        use test::Bencher;
        use std::hint::black_box;

        const TEST_VALUE: F = F::new([!W::ZERO; F::NWORDS]);

        #[bench]
        fn bench_mul(b: &mut Bencher) {
            let x = TEST_VALUE;
            let y = TEST_VALUE;
            b.iter(|| black_box(x) * &black_box(y));
        }

        #[bench]
        fn bench_mul_as_add(b: &mut Bencher) {
            let x = TEST_VALUE;
            let y = TEST_VALUE;
            b.iter(|| black_box(x).mul_as_add(&black_box(y)));
        }

        #[bench]
        fn bench_mul_fused_carry(b: &mut Bencher) {
            let x = TEST_VALUE;
            let y = TEST_VALUE;
            b.iter(|| black_box(x).mul_fused_carry(&black_box(y)));
        }

        #[bench]
        fn bench_invert(b: &mut Bencher) {
            let x = TEST_VALUE;
            b.iter(|| black_box(x).invert());
        }

        #[bench]
        fn bench_shl1(b: &mut Bencher) {
            let x = TEST_VALUE;
            b.iter(|| black_box(x).shl1_ret());
        }

        #[bench]
        fn bench_shlt(b: &mut Bencher) {
            let x = TEST_VALUE;
            b.iter(|| black_box(x).shlt_ret());
        }
    }

    for_all_fast! {
        #[test]
        fn mul_is_associative() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    for &z in &values {
                        assert_eq!((x * &y) * &z, x * &(y * &z));
                    }
                }
            }
        }

        #[test]
        fn mul_is_distributive() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    for &z in &values {
                        assert_eq!(x * &(y + z), x * &y + x * &z);
                    }
                }
            }
        }
    }

    for_all_clmul! {
        use super::super::Word;
        type W = <F as super::super::FieldExt>::W;

        #[test]
        fn mul_as_add_is_mul_clmul() {
            let values = F::get_test_values();
            for &x in &values {
                for &y in &values {
                    assert_eq!(x.mul_as_add(&y), super::super::mul_clmul_u64(&x, &y));
                }
            }
        }

        use test::Bencher;
        use std::hint::black_box;

        const TEST_VALUE: F = F::new([!W::ZERO; F::NWORDS]);

        #[bench]
        fn bench_mul_clmul(b: &mut Bencher) {
            let x = TEST_VALUE;
            let y = TEST_VALUE;
            b.iter(|| super::super::mul_clmul_u64(&black_box(x), &black_box(y)));
        }
    }
}
