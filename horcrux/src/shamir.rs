//! Implementation of the Shamir's Secret Sharing scheme.

use crate::field::Field;
use rand::thread_rng;
#[cfg(feature = "parse")]
use regex::Regex;
use std::fmt::{Debug, Display};

/// Trait to obtain the x coordinate of a share.
pub trait GetX<X: Copy> {
    /// Returns the x coordinate of a share.
    fn getx(self) -> X;
}

/// Trait for types implementing Shamir's Secret Sharing.
pub trait Shamir<F: Field> {
    /// Type for the x coordinate of shares.
    type X: Copy + From<u8>;
    /// Type for shares split from the secret.
    type Share: Copy + Debug + PartialEq + GetX<Self::X>;

    /// Create a share from its base components
    fn share(x: Self::X, y: F) -> Self::Share;

    /// Splits a secret into n shares, with k shares being sufficient to reconstruct it.
    fn split(secret: &F, k: usize, n: usize) -> Vec<Self::Share>;

    /// Reconstructs a secret from a set of shares, given the threshold parameter k. Returns `None`
    /// if reconstruction failed.
    fn reconstruct(shares: &[Self::Share], k: usize) -> Option<F>;

    /// Reconstructs a share at some x coordinate, given a set of shares and the threshold parameter
    /// k. Returns `None` if reconstruction failed.
    fn reconstruct_at(shares: &[Self::Share], k: usize, x: Self::X) -> Option<Self::Share>;

    /// Parses a share's x coordinate from a string. Returns `None` if the parsing fails.
    #[cfg(feature = "parse")]
    fn parse_x(s: &str) -> Option<Self::X>;
    /// Parses a share from a string. Returns `None` if the parsing fails.
    #[cfg(feature = "parse")]
    fn parse_share(s: &str) -> Option<Self::Share>;
}

/// Instance of `Shamir` using compact shares.
pub struct CompactShamir;
/// Instance of `Shamir` using randomized shares.
pub struct RandomShamir;

/// Representation of a share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Share<X, Y> {
    x: X,
    y: Y,
}

impl<X, Y> Display for Share<X, Y>
where
    X: Display,
    Y: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{}|{}", self.x, self.y))
    }
}

impl<X: Copy, Y> GetX<X> for Share<X, Y> {
    fn getx(self) -> X {
        self.x
    }
}

type CompactShare<F> = Share<u8, F>;
type RandomShare<F> = Share<F, F>;

fn check_split_parameters(k: usize, n: usize) {
    debug_assert!(k != 0);
    debug_assert!(n != 0);
    debug_assert!(k <= n);
    debug_assert!(n < 256);
}

fn check_reconstruct_parameters<X, Y>(shares: &[Share<X, Y>], k: usize)
where
    X: Debug + PartialEq,
    Y: Debug,
{
    debug_assert!(k != 0);
    debug_assert!(k < 256);
    debug_assert!(shares.len() >= k);
    for (i, s) in shares.iter().enumerate() {
        for (j, t) in shares.iter().enumerate() {
            if i != j {
                debug_assert!(s.x != t.x);
            }
        }
    }
}

fn generate_polynom<F: Field + Debug + Display>(secret: &F, k: usize) -> Vec<F> {
    let mut rng = thread_rng();

    let mut polynom = Vec::with_capacity(k);
    println!("Polynom = {secret}");
    for i in 1..k {
        polynom.push(F::uniform(&mut rng));
        println!("    + {} x^{i}", polynom.last().unwrap());
    }

    polynom
}

impl<F: Field + Debug + Display> Shamir<F> for CompactShamir {
    type X = u8;
    type Share = CompactShare<F>;

    fn share(x: Self::X, y: F) -> Self::Share {
        Self::Share { x, y }
    }

    fn split(secret: &F, k: usize, n: usize) -> Vec<Self::Share> {
        check_split_parameters(k, n);

        let polynom = generate_polynom(secret, k);

        let mut shares: Vec<Self::Share> = Vec::with_capacity(n);
        for i in 1..=(n as u8) {
            let x = F::from(i);

            let mut y = *secret;
            let mut xn = x;
            for p in &polynom {
                y += &(xn * p);
                xn = xn * &x;
            }

            shares.push(Self::Share { x: i, y })
        }

        shares
    }

    fn reconstruct(shares: &[Self::Share], k: usize) -> Option<F> {
        check_reconstruct_parameters(shares, k);

        let gfx: Vec<F> = shares.iter().map(|share| F::from(share.x)).collect();

        let mut secret = F::ZERO;
        for (i, si) in shares.iter().take(k).enumerate() {
            let mut lagrange = F::ONE;
            let mut denom = F::ONE;
            let xi = si.x;
            for (j, sj) in shares.iter().take(k).enumerate() {
                if j != i {
                    let xj = sj.x;
                    lagrange = lagrange * &gfx[j];
                    denom = denom * &F::from_diff(xj, xi);
                }
            }
            secret += &(lagrange * &si.y * &denom.invert());
        }

        // TODO: Verify the remaining shares.

        Some(secret)
    }

    fn reconstruct_at(shares: &[Self::Share], k: usize, x: u8) -> Option<Self::Share> {
        check_reconstruct_parameters(shares, k);

        let mut y = F::ZERO;
        for (i, si) in shares.iter().take(k).enumerate() {
            let mut lagrange = F::ONE;
            let mut denom = F::ONE;
            let xi = si.x;
            for (j, sj) in shares.iter().take(k).enumerate() {
                if j != i {
                    let xj = sj.x;
                    lagrange = lagrange * &F::from_diff(xj, x);
                    denom = denom * &F::from_diff(xj, xi);
                }
            }
            y += &(lagrange * &si.y * &denom.invert());
        }

        // TODO: Verify the remaining shares.

        Some(Self::Share { x, y })
    }

    #[cfg(feature = "parse")]
    fn parse_x(s: &str) -> Option<Self::X> {
        s.parse::<u8>().ok()
    }

    #[cfg(feature = "parse")]
    fn parse_share(s: &str) -> Option<Self::Share> {
        let regex = Regex::new(r"^([0-9]+)\|([0-9a-fA-F]+)$").unwrap();
        let captures = regex.captures(s)?;

        let x: u8 = captures[1].parse().ok()?;
        let y = F::from_bytes(&hex::decode(&captures[2]).ok()?)?;

        Some(Self::Share { x, y })
    }
}

impl<F: Field + Debug + Display> Shamir<F> for RandomShamir {
    type X = F;
    type Share = RandomShare<F>;

    fn share(x: Self::X, y: F) -> Self::Share {
        Self::Share { x, y }
    }

    fn split(secret: &F, k: usize, n: usize) -> Vec<Self::Share> {
        check_split_parameters(k, n);

        let polynom = generate_polynom(secret, k);
        let mut rng = thread_rng();

        let mut shares: Vec<Self::Share> = Vec::with_capacity(n);
        for _ in 0..n {
            let x = 'retry: loop {
                let x = F::uniform(&mut rng);
                if x == F::ZERO {
                    continue 'retry;
                }
                for s in &shares {
                    if x == s.x {
                        continue 'retry;
                    }
                }
                break x;
            };

            let mut y = *secret;
            let mut xn = x;
            for p in &polynom {
                y += &(xn * p);
                xn = xn * &x;
            }

            shares.push(Self::Share { x, y })
        }

        shares
    }

    fn reconstruct(shares: &[Self::Share], k: usize) -> Option<F> {
        check_reconstruct_parameters(shares, k);

        let mut secret = F::ZERO;
        for (i, si) in shares.iter().take(k).enumerate() {
            let mut lagrange = F::ONE;
            let mut denom = F::ONE;
            let xi = si.x;
            for (j, sj) in shares.iter().take(k).enumerate() {
                if j != i {
                    let xj = &sj.x;
                    lagrange = lagrange * xj;
                    denom = denom * &(*xj - xi);
                }
            }
            secret += &(lagrange * &si.y * &denom.invert());
        }

        // TODO: Verify the remaining shares.

        Some(secret)
    }

    fn reconstruct_at(shares: &[Self::Share], k: usize, x: F) -> Option<Self::Share> {
        check_reconstruct_parameters(shares, k);

        let mut y = F::ZERO;
        for (i, si) in shares.iter().take(k).enumerate() {
            let mut lagrange = F::ONE;
            let mut denom = F::ONE;
            let xi = si.x;
            for (j, sj) in shares.iter().take(k).enumerate() {
                if j != i {
                    let xj = sj.x;
                    lagrange = lagrange * &(xj - x);
                    denom = denom * &(xj - xi);
                }
            }
            y += &(lagrange * &si.y * &denom.invert());
        }

        // TODO: Verify the remaining shares.

        Some(Self::Share { x, y })
    }

    #[cfg(feature = "parse")]
    fn parse_x(s: &str) -> Option<Self::X> {
        F::from_bytes(&hex::decode(s).ok()?)
    }

    #[cfg(feature = "parse")]
    fn parse_share(s: &str) -> Option<Self::Share> {
        let regex = Regex::new(r"^([0-9a-fA-F]+)\|([0-9a-fA-F]+)$").unwrap();
        let captures = regex.captures(s)?;

        let x = F::from_bytes(&hex::decode(&captures[1]).ok()?)?;
        let y = F::from_bytes(&hex::decode(&captures[2]).ok()?)?;

        Some(Self::Share { x, y })
    }
}

#[cfg(test)]
mod test {
    use super::GetX;
    use super::Shamir;
    use crate::field::Field;
    use rand::thread_rng;
    use std::fmt::Debug;

    macro_rules! for_shamir {
        ( $field:ident, $mod:ident, $shamir:ident, $($tests:tt)* ) => {
            mod $mod {
                type F = crate::gf2n::$field;
                type S = super::super::super::$shamir;
                $($tests)*
            }
        }
    }

    macro_rules! for_field {
        ( $mod:ident, $field:ident, $($tests:tt)* ) => {
            mod $mod {
                for_shamir!($field, compact, CompactShamir, $($tests)*);
                for_shamir!($field, random, RandomShamir, $($tests)*);
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

    for_all! {
        #[test]
        fn can_split() {
            super::super::can_split::<F, S>();
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn can_split_big() {
            super::super::can_split_big::<F, S>();
        }

        #[test]
        fn can_reconstruct() {
            super::super::can_reconstruct::<F, S>();
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn can_reconstruct_big() {
            super::super::can_reconstruct_big::<F, S>();
        }

        #[test]
        fn can_reconstruct_pairs() {
            super::super::can_reconstruct_pairs::<F, S>();
        }

        #[test]
        fn can_reconstruct_triples() {
            super::super::can_reconstruct_triples::<F, S>();
        }

        #[test]
        fn can_reconstruct_overfit() {
            super::super::can_reconstruct_overfit::<F, S>();
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn can_reconstruct_big_pairs() {
            super::super::can_reconstruct_big_pairs::<F, S>();
        }

        #[test]
        fn can_reconstruct_at_pairs() {
            super::super::can_reconstruct_at_pairs::<F, S>();
        }

        use test::Bencher;

        #[bench]
        fn bench_split_10(b: &mut Bencher) {
            super::super::bench_split::<F, S>(b, 10, 10);
        }

        #[bench]
        fn bench_reconstruct_10(b: &mut Bencher) {
            super::super::bench_reconstruct::<F, S>(b, 10, 10);
        }

        #[bench]
        fn bench_reconstruct_at_10(b: &mut Bencher) {
            super::super::bench_reconstruct_at::<F, S>(b, 10, 10);
        }

        #[bench]
        fn bench_split_10_20(b: &mut Bencher) {
            super::super::bench_split::<F, S>(b, 10, 20);
        }

        #[bench]
        fn bench_reconstruct_10_20(b: &mut Bencher) {
            super::super::bench_reconstruct::<F, S>(b, 10, 20);
        }

        #[bench]
        fn bench_reconstruct_at_10_20(b: &mut Bencher) {
            super::super::bench_reconstruct_at::<F, S>(b, 10, 20);
        }

        #[bench]
        fn bench_reconstruct_10_20_arbitrary(b: &mut Bencher) {
            super::super::bench_reconstruct_arbitrary::<F, S>(b, 10, 20);
        }

        #[bench]
        fn bench_reconstruct_at_10_20_arbitrary(b: &mut Bencher) {
            super::super::bench_reconstruct_at_arbitrary::<F, S>(b, 10, 20);
        }

        #[bench]
        fn bench_split_big_triple(b: &mut Bencher) {
            super::super::bench_split::<F, S>(b, 3, 255);
        }

        #[bench]
        fn bench_reconstruct_big_triple(b: &mut Bencher) {
            super::super::bench_reconstruct::<F, S>(b, 3, 255);
        }

        #[bench]
        fn bench_reconstruct_at_big_triple(b: &mut Bencher) {
            super::super::bench_reconstruct_at::<F, S>(b, 3, 255);
        }

        #[bench]
        fn bench_reconstruct_big_triple_arbitrary(b: &mut Bencher) {
            super::super::bench_reconstruct_arbitrary::<F, S>(b, 3, 255);
        }

        #[bench]
        fn bench_reconstruct_at_big_triple_arbitrary(b: &mut Bencher) {
            super::super::bench_reconstruct_at_arbitrary::<F, S>(b, 3, 255);
        }
    }

    for_all_fast! {
        #[test]
        fn can_reconstruct_big_triples() {
            super::super::can_reconstruct_big_triples::<F, S>();
        }

        use test::Bencher;

        #[bench]
        fn bench_split_big_all(b: &mut Bencher) {
            super::super::bench_split::<F, S>(b, 255, 255);
        }

        #[bench]
        fn bench_reconstruct_big_all(b: &mut Bencher) {
            super::super::bench_reconstruct::<F, S>(b, 255, 255);
        }

        #[bench]
        fn bench_reconstruct_at_big_all(b: &mut Bencher) {
            super::super::bench_reconstruct_at::<F, S>(b, 255, 255);
        }
    }

    fn can_split<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const KMAX: usize = 5;
        #[cfg(debug_assertions)]
        const KMAX: usize = 2;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for k in 1..KMAX {
            for n in k..=255 {
                let shares = S::split(&secret, k, n);
                assert_eq!(shares.len(), n);
            }
        }
    }

    fn can_reconstruct<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const NMAX: usize = 10;
        #[cfg(debug_assertions)]
        const NMAX: usize = 5;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for k in 1..NMAX {
            for n in k..NMAX {
                let shares = S::split(&secret, k, n);
                let reconstructed = S::reconstruct(&shares, k);
                assert_eq!(reconstructed, Some(secret));
            }
        }
    }

    fn can_reconstruct_pairs<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const NMAX: usize = 20;
        #[cfg(debug_assertions)]
        const NMAX: usize = 5;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for n in 2..NMAX {
            reconstruct_pairs::<F, S>(secret, n);
        }
    }

    fn can_reconstruct_triples<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const NMAX: usize = 10;
        #[cfg(debug_assertions)]
        const NMAX: usize = 5;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for n in 3..NMAX {
            reconstruct_triples::<F, S>(secret, n);
        }
    }

    fn can_reconstruct_overfit<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const NMAX: usize = 10;
        #[cfg(debug_assertions)]
        const NMAX: usize = 5;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for k in 1..NMAX {
            for n in k..NMAX {
                let shares = S::split(&secret, k, n);
                let reconstructed = S::reconstruct(&shares, n);
                assert_eq!(reconstructed, Some(secret));
            }
        }
    }

    fn can_reconstruct_at_pairs<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        #[cfg(not(debug_assertions))]
        const NMAX: usize = 10;
        #[cfg(debug_assertions)]
        const NMAX: usize = 5;
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        for n in 3..NMAX {
            reconstruct_at_pairs::<F, S>(secret, n);
        }
    }

    #[cfg(not(debug_assertions))]
    fn can_split_big<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        let shares = S::split(&secret, 255, 255);
        assert_eq!(shares.len(), 255);
    }

    #[cfg(not(debug_assertions))]
    fn can_reconstruct_big<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        let shares = S::split(&secret, 255, 255);
        let reconstructed = S::reconstruct(&shares, 255);
        assert_eq!(reconstructed, Some(secret));
    }

    #[cfg(not(debug_assertions))]
    fn can_reconstruct_big_pairs<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        reconstruct_pairs::<F, S>(secret, 255);
    }

    #[cfg(not(debug_assertions))]
    fn can_reconstruct_big_triples<F: Field + Debug, S: Shamir<F> + ?Sized>() {
        let mut rng = thread_rng();
        let secret = F::uniform(&mut rng);
        reconstruct_triples::<F, S>(secret, 255);
    }

    fn reconstruct_pairs<F: Field + Debug, S: Shamir<F> + ?Sized>(secret: F, n: usize) {
        let shares = S::split(&secret, 2, n);
        for a in 0..n {
            for b in 0..a {
                let reconstructed = S::reconstruct(&[shares[a], shares[b]], 2);
                assert_eq!(reconstructed, Some(secret));
            }
        }
    }

    fn reconstruct_triples<F: Field + Debug, S: Shamir<F> + ?Sized>(secret: F, n: usize) {
        let shares = S::split(&secret, 3, n);
        for a in 0..n {
            for b in 0..a {
                for c in 0..b {
                    let reconstructed = S::reconstruct(&[shares[a], shares[b], shares[c]], 3);
                    assert_eq!(reconstructed, Some(secret));
                }
            }
        }
    }

    fn reconstruct_at_pairs<F: Field + Debug, S: Shamir<F> + ?Sized>(secret: F, n: usize) {
        let shares = S::split(&secret, 2, n);
        for a in 0..n {
            for b in 0..a {
                for c in 0..b {
                    let reconstructed =
                        S::reconstruct_at(&[shares[a], shares[b]], 2, shares[c].getx());
                    assert_eq!(reconstructed, Some(shares[c]));
                }
            }
        }
    }

    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    use std::hint::black_box;
    use test::Bencher;

    fn bench_split<F: Field + Debug, S: Shamir<F> + ?Sized>(b: &mut Bencher, k: usize, n: usize) {
        let secret = F::uniform(&mut thread_rng());
        b.iter(|| S::split(black_box(&secret), k, n));
    }

    fn bench_reconstruct<F: Field + Debug, S: Shamir<F> + ?Sized>(
        b: &mut Bencher,
        k: usize,
        n: usize,
    ) {
        let secret = F::uniform(&mut thread_rng());
        let shares = S::split(&secret, k, n);
        b.iter(|| S::reconstruct(black_box(&shares), k));
    }

    fn bench_reconstruct_arbitrary<F: Field + std::fmt::Debug, S: Shamir<F> + ?Sized>(
        b: &mut Bencher,
        k: usize,
        n: usize,
    ) {
        let secret = F::uniform(&mut thread_rng());
        let mut shares = S::split(&secret, k, n);
        // Choose a fast rng to limit the impact on the measurements.
        let mut rng = SmallRng::from_entropy();
        b.iter(|| {
            // Pick arbitrary values on each run.
            let (chosen, _) = shares.partial_shuffle(&mut rng, k);
            S::reconstruct(black_box(&chosen), k)
        });
    }

    fn bench_reconstruct_at<F: Field + Debug, S: Shamir<F> + ?Sized>(
        b: &mut Bencher,
        k: usize,
        n: usize,
    ) {
        let secret = F::uniform(&mut thread_rng());
        let shares = S::split(&secret, k, n);
        let x = S::X::from(42);
        b.iter(|| S::reconstruct_at(black_box(&shares), k, black_box(x)));
    }

    fn bench_reconstruct_at_arbitrary<F: Field + std::fmt::Debug, S: Shamir<F> + ?Sized>(
        b: &mut Bencher,
        k: usize,
        n: usize,
    ) {
        let secret = F::uniform(&mut thread_rng());
        let mut shares = S::split(&secret, k, n);
        // Choose a fast rng to limit the impact on the measurements.
        let mut rng = SmallRng::from_entropy();
        let x = S::X::from(42);
        b.iter(|| {
            // Pick arbitrary values on each run.
            let (chosen, _) = shares.partial_shuffle(&mut rng, k);
            S::reconstruct_at(black_box(&chosen), k, black_box(x))
        });
    }
}
