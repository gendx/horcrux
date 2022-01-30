#![feature(bench_black_box, test, const_fn_trait_bound)]
#![deny(missing_docs)]

//! Rust implementation of Shamir's Secret Sharing.

#[cfg(test)]
extern crate test;

pub mod field;
pub mod gf2n;
pub mod shamir;
