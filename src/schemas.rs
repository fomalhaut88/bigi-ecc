extern crate rand;

use rand::Rng;
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS, BIGI_BITS};
use crate::{point};
use crate::base::{Point, CurveTrait};
use crate::weierstrass::WeierstrassCurve;
use crate::montgomery::MontgomeryCurve;
use crate::edwards::EdwardsCurve;


pub struct Schema<T: CurveTrait> {
    pub bits: usize,
    pub title: &'static str,
    pub curve: T,
    pub order: Bigi,
    pub cofactor: Bigi,
    pub generator: Point
}


impl<T: CurveTrait> Schema<T> {
    pub fn get_point(&self, k: &Bigi) -> Point {
        self.curve.mul(&self.generator, k)
    }

    pub fn generate_pair<R: Rng + ?Sized>(&self, rng: &mut R) -> (Bigi, Point) {
        let x = Bigi::gen_random(rng, self.bits, false) % &self.order;
        let h = self.get_point(&x);
        (x, h)
    }
}


pub fn load_secp256k1() -> Schema<WeierstrassCurve> {
    assert!(BIGI_BITS >= 2 * 256);
    Schema {
        bits: 256,
        title: "secp256k1",
        curve: WeierstrassCurve {
            a: bigi![0],
            b: bigi![7],
            m: Bigi::from_hex("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
        },
        order: Bigi::from_hex("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"),
        cofactor: bigi![1],
        generator: point!(
            Bigi::from_hex("0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
            Bigi::from_hex("0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8")
        )
    }
}


pub fn load_fp254bnb() -> Schema<WeierstrassCurve> {
    assert!(BIGI_BITS >= 2 * 254);
    Schema {
        bits: 254,
        title: "fp254bnb",
        curve: WeierstrassCurve {
            a: bigi![0],
            b: bigi![2],
            m: Bigi::from_hex("0x2523648240000001BA344D80000000086121000000000013A700000000000013")
        },
        order: Bigi::from_hex("0x2523648240000001BA344D8000000007FF9F800000000010A10000000000000D"),
        cofactor: bigi![1],
        generator: point!(
            Bigi::from_hex("0x2523648240000001BA344D80000000086121000000000013A700000000000012"),
            Bigi::from_hex("0x1")
        )
    }
}


pub fn load_curve25519() -> Schema<MontgomeryCurve> {
    assert!(BIGI_BITS >= 2 * 255);
    Schema {
        bits: 255,
        title: "curve25519",
        curve: MontgomeryCurve {
            a: bigi![486662],
            b: bigi![1],
            m: Bigi::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFED")
        },
        order: Bigi::from_hex("0x1000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED"),
        cofactor: bigi![8],
        generator: point!(
            Bigi::from_hex("0x9"),
            Bigi::from_hex("0x20AE19A1B8A086B4E01EDD2C7748D14C923D4D7E6D7C61B229E9C5A27ECED3D9")
        )
    }
}


pub fn load_curve1174() -> Schema<EdwardsCurve> {
    assert!(BIGI_BITS >= 2 * 251);
    Schema {
        bits: 251,
        title: "curve1174",
        curve: EdwardsCurve {
            d: Bigi::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFB61"),
            m: Bigi::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7")
        },
        order: Bigi::from_hex("0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF77965C4DFD307348944D45FD166C971"),
        cofactor: bigi![4],
        generator: point!(
            Bigi::from_hex("0x37FBB0CEA308C479343AEE7C029A190C021D96A492ECD6516123F27BCE29EDA"),
            Bigi::from_hex("0x6B72F82D47FB7CC6656841169840E0C4FE2DEE2AF3F976BA4CCB1BF9B46360E")
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_load_secp256k1(b: &mut Bencher) {
        b.iter(|| load_secp256k1());
    }

    #[bench]
    fn bench_load_fp254bnb(b: &mut Bencher) {
        b.iter(|| load_fp254bnb());
    }

    #[bench]
    fn bench_load_curve25519(b: &mut Bencher) {
        b.iter(|| load_curve25519());
    }

    #[bench]
    fn bench_load_curve1174(b: &mut Bencher) {
        b.iter(|| load_curve1174());
    }
}
