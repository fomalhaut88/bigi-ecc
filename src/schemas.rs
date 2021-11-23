extern crate rand;

use rand::Rng;
use bigi::Bigi;
use crate::{point};
use crate::base::{Point, CurveTrait};
use crate::weierstrass::WeierstrassCurve;
use crate::montgomery::MontgomeryCurve;
use crate::edwards::EdwardsCurve;


/// A struct for ECC schema that includes the curve, its order, cofactor and
/// a generator point.
pub struct Schema<T, const N: usize> where T: CurveTrait<N> {
    pub bits: usize,
    pub title: &'static str,
    pub curve: T,
    pub order: Bigi<N>,
    pub cofactor: Bigi<N>,
    pub generator: Point<N>
}


impl<T, const N: usize> Schema<T, N> where T: CurveTrait<N> {
    /// Gets point `k * G` where `G` is a generator.
    pub fn get_point(&self, k: &Bigi<N>) -> Point<N> {
        self.curve.mul(&self.generator, k)
    }

    /// Gets a random point on the curve.
    pub fn generate_pair<R: Rng + ?Sized>(&self, rng: &mut R
                ) -> (Bigi<N>, Point<N>) {
        let x = Bigi::<N>::gen_random(rng, self.bits, false) % &self.order;
        let h = self.get_point(&x);
        (x, h)
    }
}


/// Returns SECP256K1 schema.
pub fn load_secp256k1() -> Schema<WeierstrassCurve<4>, 4> {
    Schema {
        bits: 256,
        title: "secp256k1",
        curve: WeierstrassCurve::<4> {
            a: Bigi::<4>::from_hex("0x0"),
            b: Bigi::<4>::from_hex("0x7"),
            m: Bigi::<4>::from_hex("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
        },
        order: Bigi::<4>::from_hex("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"),
        cofactor: Bigi::<4>::from_hex("0x1"),
        generator: point!(
            Bigi::<4>::from_hex("0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
            Bigi::<4>::from_hex("0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8")
        )
    }
}


/// Returns FP254BNB schema.
pub fn load_fp254bnb() -> Schema<WeierstrassCurve<4>, 4> {
    Schema {
        bits: 254,
        title: "fp254bnb",
        curve: WeierstrassCurve::<4> {
            a: Bigi::<4>::from_hex("0x0"),
            b:Bigi::<4>::from_hex("0x2"),
            m: Bigi::<4>::from_hex("0x2523648240000001BA344D80000000086121000000000013A700000000000013")
        },
        order: Bigi::<4>::from_hex("0x2523648240000001BA344D8000000007FF9F800000000010A10000000000000D"),
        cofactor: Bigi::<4>::from_hex("0x1"),
        generator: point!(
            Bigi::<4>::from_hex("0x2523648240000001BA344D80000000086121000000000013A700000000000012"),
            Bigi::<4>::from_hex("0x1")
        )
    }
}


/// Returns Curve25519 schema.
pub fn load_curve25519() -> Schema<MontgomeryCurve<4>, 4> {
    Schema {
        bits: 255,
        title: "curve25519",
        curve: MontgomeryCurve::<4> {
            a: Bigi::<4>::from_hex("0x76D06"),
            b: Bigi::<4>::from_hex("0x1"),
            m: Bigi::<4>::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFED")
        },
        order: Bigi::<4>::from_hex("0x1000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED"),
        cofactor: Bigi::<4>::from_hex("0x8"),
        generator: point!(
            Bigi::<4>::from_hex("0x9"),
            Bigi::<4>::from_hex("0x20AE19A1B8A086B4E01EDD2C7748D14C923D4D7E6D7C61B229E9C5A27ECED3D9")
        )
    }
}


pub fn load_curve1174() -> Schema<EdwardsCurve<4>, 4> {
    Schema {
        bits: 251,
        title: "curve1174",
        curve: EdwardsCurve::<4> {
            d: Bigi::<4>::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFB61"),
            m: Bigi::<4>::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7")
        },
        order: Bigi::<4>::from_hex("0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF77965C4DFD307348944D45FD166C971"),
        cofactor: Bigi::<4>::from_hex("0x4"),
        generator: point!(
            Bigi::<4>::from_hex("0x37FBB0CEA308C479343AEE7C029A190C021D96A492ECD6516123F27BCE29EDA"),
            Bigi::<4>::from_hex("0x6B72F82D47FB7CC6656841169840E0C4FE2DEE2AF3F976BA4CCB1BF9B46360E")
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_load_secp256k1(bencher: &mut Bencher) {
        bencher.iter(|| load_secp256k1());
    }

    #[bench]
    fn bench_load_fp254bnb(bencher: &mut Bencher) {
        bencher.iter(|| load_fp254bnb());
    }

    #[bench]
    fn bench_load_curve25519(bencher: &mut Bencher) {
        bencher.iter(|| load_curve25519());
    }

    #[bench]
    fn bench_load_curve1174(bencher: &mut Bencher) {
        bencher.iter(|| load_curve1174());
    }
}
