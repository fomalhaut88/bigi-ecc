extern crate rand;

use rand::Rng;
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use crate::{point};
use crate::base::{Point, CurveTrait};
use crate::weierstrass::WeierstrassCurve;


pub struct Schema<T: CurveTrait> {
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

    pub fn generate_pair<R: Rng + ?Sized>(&self, bits: usize, rng: &mut R) -> (Bigi, Point) {
        let x = Bigi::gen_random(rng, bits, false) % &self.order;
        let h = self.get_point(&x);
        (x, h)
    }
}


pub fn load_secp256k1() -> Schema<WeierstrassCurve> {
    Schema {
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
