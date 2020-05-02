extern crate rand;

use rand::Rng;
use bigi::Bigi;
use crate::base::{Point, CurveTrait};
use crate::schemas::Schema;


pub struct PrivateKey<'a, T: CurveTrait> {
    bits: usize,
    schema: &'a Schema<T>,
    pub h: Point,
    pub x: Bigi
}


pub struct PublicKey<'a, T: CurveTrait> {
    bits: usize,
    schema: &'a Schema<T>,
    h: Point
}


impl<'a, T: CurveTrait> PrivateKey<'a, T> {
    pub fn new<R: Rng + ?Sized>(bits: usize, rng: &mut R,
                                schema: &'a Schema<T>) -> Self {
        let (x, h) = schema.generate_pair(bits, rng);
        PrivateKey {
            bits: bits,
            schema: schema,
            h: h,
            x: x
        }
    }

    pub fn get_public_key(&self) -> PublicKey<'a, T> {
        PublicKey {
            bits: self.bits,
            schema: self.schema,
            h: self.h
        }
    }

    pub fn decrypt(&self, encrypted: &(Point, Vec<Point>)) -> Vec<Point> {
        let (c1, c2) = encrypted;
        let s = self.schema.curve.mul(&c1, &self.x);
        let si = self.schema.curve.inv(&s);
        c2.iter().map(|m| {
            self.schema.curve.add(&si, &m)
        }).collect()
    }
}


impl<'a, T: CurveTrait> PublicKey<'a, T> {
    pub fn new(bits: usize, schema: &'a Schema<T>, h: &Point) -> Self {
        PublicKey {
            bits: bits,
            schema: schema,
            h: *h
        }
    }

    pub fn encrypt<R: Rng + ?Sized>(&self, points: &Vec<Point>,
                                    rng: &mut R) -> (Point, Vec<Point>) {
        let (y, c1) = self.schema.generate_pair(self.bits, rng);
        let s = self.schema.curve.mul(&self.h, &y);
        let c2 = points.iter().map(|m| {
            self.schema.curve.add(&s, &m)
        }).collect();
        (c1, c2)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{point, schemas};

    #[test]
    fn test_elgamal() {
        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let private_key = PrivateKey::new(256, &mut rng, &schema);
        let public_key = private_key.get_public_key();

        let mx = Bigi::from_hex("0x3541E1F95455C55319AC557D1ECC817C227CBE68405E78838577B99FE7E02D2B");
        let my = schema.curve.find_y(&mx).unwrap().0;
        let original = vec![
            point![mx, my]
        ];

        let encrypted = public_key.encrypt(&original, &mut rng);
        let decripted = private_key.decrypt(&encrypted);

        assert_eq!(original[0], decripted[0]);
    }
}
