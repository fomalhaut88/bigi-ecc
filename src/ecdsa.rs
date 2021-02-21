extern crate rand;

use rand::Rng;
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use bigi::prime::{add_mod, mul_mod, div_mod, inv_mod};
use crate::base::{CurveTrait, Point};
use crate::schemas::{Schema};


pub fn build_signature<T: CurveTrait, R: Rng + ?Sized>(
            rng: &mut R,
            schema: &Schema<T>,
            private_key: &Bigi,
            hash: &Vec<u8>
        ) -> (Bigi, Bigi) {
    let h = Bigi::from_bytes(hash) % &schema.order;

    let (k, r) = {
        let mut k;
        let mut r;
        loop {
            let pair = schema.generate_pair(rng);
            k = pair.0;
            r = pair.1.x % &schema.order;
            if r != bigi![0] {
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


pub fn check_signature<T: CurveTrait>(
            schema: &Schema<T>,
            public_key: &Point,
            hash: &Vec<u8>,
            signature: &(Bigi, Bigi)
        ) -> bool {
    let h = Bigi::from_bytes(hash) % &schema.order;
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
    use test::Bencher;
    use sha2::{Sha256, Digest};
    use crate::schemas;

    #[test]
    fn test_ecdsa() {
        let message = b"This project is sort of half polyfill for features like the host bindings proposal and half features for empowering high-level interactions between JS and wasm-compiled code (currently mostly from Rust). More specifically this project allows JS/wasm to communicate with strings, JS objects, classes, etc, as opposed to purely integers and floats. Using wasm-bindgen for example you can define a JS class in Rust or take a string from JS or return one. The functionality is growing as well!";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.input(&message[..]);
        let hash = hasher.result();

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
                            &(bigi![1231], bigi![3246457])),
            false
        );

        assert_eq!(
            check_signature(&schema, &public_key, &hash.to_vec(),
                            &(bigi![0], bigi![0])),
            false
        );
    }

    #[bench]
    fn bench_build_signature(b: &mut Bencher) {
        let message = b"This project is sort of half polyfill for features like the host bindings proposal and half features for empowering high-level interactions between JS and wasm-compiled code (currently mostly from Rust). More specifically this project allows JS/wasm to communicate with strings, JS objects, classes, etc, as opposed to purely integers and floats. Using wasm-bindgen for example you can define a JS class in Rust or take a string from JS or return one. The functionality is growing as well!";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.input(&message[..]);
        let hash = hasher.result();

        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();
        let (private_key, _public_key) = schema.generate_pair(&mut rng);

        b.iter(|| build_signature(
            &mut rng, &schema, &private_key, &hash.to_vec()
        ));
    }

    #[bench]
    fn bench_check_signature(b: &mut Bencher) {
        let message = b"This project is sort of half polyfill for features like the host bindings proposal and half features for empowering high-level interactions between JS and wasm-compiled code (currently mostly from Rust). More specifically this project allows JS/wasm to communicate with strings, JS objects, classes, etc, as opposed to purely integers and floats. Using wasm-bindgen for example you can define a JS class in Rust or take a string from JS or return one. The functionality is growing as well!";

        let mut hasher = Sha256::new();
        hasher.reset();
        hasher.input(&message[..]);
        let hash = hasher.result();

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
