//! This module implements [ECDSA](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm)
//! algorithms.
//!
//! Usage example:
//! ```rust
//! use sha2::{Sha256, Digest};
//! use bigi_ecc::schemas;
//! use bigi_ecc::ecdsa::{build_signature, check_signature};
//!
//! let msg = b"a test phrase";
//!
//! // Get SHA256 hash of the message
//! let mut hasher = Sha256::new();
//! hasher.reset();
//! hasher.update(&msg[..]);
//! let hash = hasher.finalize();
//!
//! // Load a crypto schema
//! let schema = schemas::load_secp256k1();
//!
//! // Generate a key pair
//! let mut rng = rand::thread_rng();
//! let (private_key, public_key) = schema.generate_pair(&mut rng);
//!
//! // Build signature
//! let signature = build_signature(
//!     &mut rng, &schema, &private_key, &hash.to_vec()
//! );
//!
//! // Chech the signature
//! assert_eq!(
//!     check_signature(&schema, &public_key, &hash.to_vec(), &signature),
//!     true
//! );
//! ```
extern crate rand;

use rand::Rng;
use bigi::Bigi;
use bigi::prime::{add_mod, mul_mod, div_mod, inv_mod};
use crate::base::{CurveTrait, Point};
use crate::schemas::Schema;


/// Builds a signature for given schema, private key and hash of a message.
pub fn build_signature<R: Rng + ?Sized, T: CurveTrait<N>, const N: usize> (
            rng: &mut R,
            schema: &Schema<T, N>,
            private_key: &Bigi<N>,
            hash: &Vec<u8>
        ) -> (Bigi<N>, Bigi<N>) {
    // let mut hash_bytes = hash.clone();
    // hash_bytes.resize(N << 3, 0);

    assert!(hash.len() == N << 2);

    let mut hash_aligned = vec![0u8; N << 3];
    hash_aligned[..hash.len()].copy_from_slice(hash);

    let h = Bigi::<N>::from_bytes(&hash_aligned) % &schema.order;

    let (k, r) = {
        let mut k;
        let mut r;
        loop {
            let pair = schema.generate_pair(rng);
            k = pair.0;
            r = pair.1.x % &schema.order;
            if r != Bigi::<N>::from(0) {
                break;
            }
        }
        (k, r)
    };

    let s = div_mod(
        &add_mod(
            &mul_mod(&private_key, &r, &schema.order),
            &h, &schema.order
        ),
        &k, &schema.order
    );

    (r, s)
}


/// Checks for the signature for the given schema, public key and the hash
/// of a message.
pub fn check_signature<T: CurveTrait<N>, const N: usize> (
            schema: &Schema<T, N>,
            public_key: &Point<N>,
            hash: &Vec<u8>,
            signature: &(Bigi<N>, Bigi<N>)
        ) -> bool {
    assert!(hash.len() == N << 2);

    let mut hash_aligned = vec![0u8; N << 3];
    hash_aligned[..hash.len()].copy_from_slice(hash);

    let h = Bigi::<N>::from_bytes(&hash_aligned) % &schema.order;
    let (r, s) = signature;

    if r.is_zero() || (r >= &schema.order) ||
            s.is_zero() || (s >= &schema.order) {
        return false;
    }

    let si = inv_mod(&s, &schema.order);
    let u1 = mul_mod(&si, &h, &schema.order);
    let u2 = mul_mod(&si, &r, &schema.order);
    let p = schema.curve.add(
        &schema.get_point(&u1),
        &schema.curve.mul(&public_key, &u2)
    );
    p.x == *r
}


#[cfg(test)]
mod tests {
    use super::*;
    use bigi::bigi;
    use test::Bencher;
    use sha2::{Sha256, Digest};
    use crate::schemas;

    #[test]
    fn test_ecdsa() {
        let message = b"a test phrase";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.update(&message[..]);
        let hash = hasher.finalize();

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();
        let (private_key, public_key) = schema.generate_pair(&mut rng);

        let signature = build_signature(
            &mut rng, &schema, &private_key, &hash.to_vec()
        );

        assert_eq!(
            check_signature(&schema, &public_key, &hash.to_vec(), &signature),
            true
        );

        assert_eq!(
            check_signature(&schema, &public_key, &hash.to_vec(),
                            &(bigi![8; 1231], bigi![8; 3246457])),
            false
        );

        assert_eq!(
            check_signature(&schema, &public_key, &hash.to_vec(),
                            &(bigi![8; 0], bigi![8; 0])),
            false
        );
    }

    #[bench]
    fn bench_build_signature(b: &mut Bencher) {
        let message = b"a test phrase";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.update(&message[..]);
        let hash = hasher.finalize();

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();
        let (private_key, _public_key) = schema.generate_pair(&mut rng);

        b.iter(|| build_signature(
            &mut rng, &schema, &private_key, &hash.to_vec()
        ));
    }

    #[bench]
    fn bench_check_signature(b: &mut Bencher) {
        let message = b"a test phrase";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.update(&message[..]);
        let hash = hasher.finalize();

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();
        let (private_key, public_key) = schema.generate_pair(&mut rng);

        let signature = build_signature(
            &mut rng, &schema, &private_key, &hash.to_vec()
        );

        b.iter(|| check_signature(
            &schema, &public_key, &hash.to_vec(), &signature)
        );
    }
}
