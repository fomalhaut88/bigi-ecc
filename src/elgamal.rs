//! This module implements [ElGamal encription](https://en.wikipedia.org/wiki/ElGamal_encryption)
//!
//! Example:
//! ```rust
//! use bigi_ecc::schemas;
//! use bigi_ecc::elgamal::{encrypt, decrypt};
//!
//! // A test phrase
//! let message = b"a test phrase";
//!
//! // Load schema
//! let schema = schemas::load_secp256k1();
//!
//! // Generate a key pair
//! let mut rng = rand::thread_rng();
//! let (private_key, public_key) = schema.generate_pair(&mut rng);
//!
//! // Encrypt the message, the result is a pair of points on the curve.
//! let encrypted = encrypt(&mut rng, &schema, &public_key, &message[..]);
//!
//! // Decrypt the message
//! let mut decripted = decrypt(&schema, &private_key, &encrypted);
//!
//! // Remove trailing zeros
//! if let Some(idx) = decripted.iter().rposition(|&c| c != 0) {
//!     decripted.truncate(idx + 1);
//! }
//!
//! assert_eq!(decripted, message);
//! ```

extern crate rand;

use rand::Rng;
use bigi::Bigi;
use crate::point;
use crate::base::{Point, CurveTrait};
use crate::schemas::Schema;


/// Encrypt `bytes` with `public_key` according to ElGamal encryption.
/// The result is a pair of points.
pub fn encrypt<R: Rng + ?Sized, T: CurveTrait<N>, const N: usize> (
            rng: &mut R,
            schema: &Schema<T, N>,
            public_key: &Point<N>,
            bytes: &[u8]
        ) -> (Point<N>, Point<N>) {
    let (y, c1) = schema.generate_pair(rng);
    let s = schema.curve.mul(&public_key, &y);
    let m = bytes_to_point(bytes, &schema.curve);
    let c2 = schema.curve.add(&s, &m);
    (c1, c2)
}


/// Decrypt a pair of points `encrypted` with `private_key` according to
/// ElGamal encryption. The result is a vector of bytes.
pub fn decrypt<T: CurveTrait<N>, const N: usize> (
            schema: &Schema<T, N>,
            private_key: &Bigi<N>,
            encrypted: &(Point<N>, Point<N>)
        ) -> Vec<u8> {
    let (c1, c2) = encrypted;
    let s = schema.curve.mul(&c1, &private_key);
    let si = schema.curve.inv(&s);
    let p = schema.curve.add(&si, &c2);
    bytes_from_point(&p)
}


fn bytes_to_point<T: CurveTrait<N>, const N: usize>(
            bytes: &[u8], curve: &T) -> Point<N> {
    assert!(bytes.len() <= (N << 3) - 2);

    let mut bytes_aligned = vec![0u8; N << 3];
    bytes_aligned[..bytes.len()].copy_from_slice(bytes);

    let mut x = Bigi::<N>::from_bytes(&bytes_aligned) << 8;
    let y;

    loop {
        let one = Bigi::<N>::from(1);
        match curve.find_y(&x) {
            Ok(roots) => { y = roots.0; break; },
            Err(_e) => { x += &one }
        }
    }

    point!(x, y)
}


fn bytes_from_point<const N: usize>(p: &Point<N>) -> Vec<u8> {
    (p.x >> 8).to_bytes()[..(N << 3) - 2].to_vec()
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use crate::schemas;

    #[test]
    fn test_elgamal() {
        let message = b"a test phrase";

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let (private_key, public_key) = schema.generate_pair(&mut rng);

        let encrypted = encrypt(&mut rng, &schema, &public_key, &message[..]);
        let mut decripted = decrypt(&schema, &private_key, &encrypted);

        if let Some(idx) = decripted.iter().rposition(|&c| c != 0) {
            decripted.truncate(idx + 1);
        }

        assert_eq!(decripted, message);
    }

    #[bench]
    fn bench_encrypt(bencher: &mut Bencher) {
        let message = b"a test phrase";

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let (_private_key, public_key) = schema.generate_pair(&mut rng);

        bencher.iter(|| encrypt(&mut rng, &schema, &public_key, &message[..]))
    }

    #[bench]
    fn bench_decrypt(bencher: &mut Bencher) {
        let message = b"a test phrase";

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let (private_key, public_key) = schema.generate_pair(&mut rng);

        let encrypted = encrypt(&mut rng, &schema, &public_key, &message[..]);

        bencher.iter(|| decrypt(&schema, &private_key, &encrypted))
    }
}
