extern crate rand;

use rand::Rng;
use bigi::Bigi;
use crate::base::{Point, CurveTrait};
use crate::schemas::Schema;


pub fn encrypt<R: Rng + ?Sized, T: CurveTrait>(
            bits: usize,
            rng: &mut R,
            schema: &Schema<T>,
            public_key: &Point,
            points: &Vec<Point>
        ) -> (Point, Vec<Point>) {
    let (y, c1) = schema.generate_pair(bits, rng);
    let s = schema.curve.mul(&public_key, &y);
    let c2 = points.iter().map(|m| {
        schema.curve.add(&s, &m)
    }).collect();
    (c1, c2)
}


pub fn decrypt<T: CurveTrait>(
            schema: &Schema<T>,
            private_key: &Bigi,
            encrypted: &(Point, Vec<Point>)
        ) -> Vec<Point> {
    let (c1, c2) = encrypted;
    let s = schema.curve.mul(&c1, &private_key);
    let si = schema.curve.inv(&s);
    c2.iter().map(|m| {
        schema.curve.add(&si, &m)
    }).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{point, schemas};

    #[test]
    fn test_elgamal() {
        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let (private_key, public_key) = schema.generate_pair(256, &mut rng);

        let mx = Bigi::from_hex("0x3541E1F95455C55319AC557D1ECC817C227CBE68405E78838577B99FE7E02D2B");
        let my = schema.curve.find_y(&mx).unwrap().0;
        let original = vec![
            point![mx, my]
        ];

        let encrypted = encrypt(256, &mut rng, &schema, &public_key, &original);
        let decripted = decrypt(&schema, &private_key, &encrypted);

        assert_eq!(original[0], decripted[0]);
    }
}
