#![feature(test)]
#![deny(missing_docs)]

//! Rust implementation of Shamir's Secret Sharing.

#[cfg(test)]
extern crate test;

pub mod field;
pub mod gf2n;
pub mod shamir;
