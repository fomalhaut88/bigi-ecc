//! **bigi-ecc** is a Rust library for
//! [elliptic-curve cryptography](https://en.wikipedia.org/wiki/Elliptic-curve_cryptography).
//! It contains the most popular elliptic curves (Weierstrass curve,
//! Montgomery curve, Edwards curve) and algorithms to encrypt and decrypt data,
//! to build signatures and to map data blocks. Also there are several certain
//! elliptic curves to import. The library is built for Rust Nightly strictly.

#![feature(test)]
extern crate test;

pub mod base;
pub mod weierstrass;
pub mod montgomery;
pub mod edwards;
pub mod schemas;
pub mod ecdsa;
pub mod elgamal;

pub use base::*;
pub use weierstrass::*;
pub use montgomery::*;
pub use edwards::*;
