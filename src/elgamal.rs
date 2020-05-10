extern crate rand;

use rand::Rng;
use bigi::Bigi;
use crate::base::{Point, CurveTrait};
use crate::schemas::Schema;


pub fn encrypt<R: Rng + ?Sized, T: CurveTrait>(
            rng: &mut R,
            schema: &Schema<T>,
            public_key: &Point,
            points: &Vec<Point>
        ) -> (Point, Vec<Point>) {
    let (y, c1) = schema.generate_pair(rng);
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
    use test::Bencher;
    use crate::{point, schemas};
    use crate::mapping::Mapper;
    use crate::schemas::load_secp256k1;

    #[test]
    fn test_elgamal() {
        let mut rng = rand::thread_rng();
        let schema = schemas::load_secp256k1();

        let (private_key, public_key) = schema.generate_pair(&mut rng);

        let mx = Bigi::from_hex("0x3541E1F95455C55319AC557D1ECC817C227CBE68405E78838577B99FE7E02D2B");
        let my = schema.curve.find_y(&mx).unwrap().0;
        let original = vec![
            point![mx, my]
        ];

        let encrypted = encrypt(&mut rng, &schema, &public_key, &original);
        let decripted = decrypt(&schema, &private_key, &encrypted);

        assert_eq!(original[0], decripted[0]);
    }

    #[bench]
    fn bench_encrypt_1024(b: &mut Bencher) {
        let body: Vec<u8> = (0..1024).map(|_| { rand::random::<u8>() }).collect();

        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let (_private_key, public_key) = schema.generate_pair(&mut rng);
        let mapper = Mapper::new(256, &schema.curve);
        let points = mapper.pack(&body);

        b.iter(|| encrypt(&mut rng, &schema, &public_key, &points));
    }

    #[bench]
    fn bench_encrypt_10240(b: &mut Bencher) {
        let body: Vec<u8> = (0..10240).map(|_| { rand::random::<u8>() }).collect();

        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let (_private_key, public_key) = schema.generate_pair(&mut rng);
        let mapper = Mapper::new(256, &schema.curve);
        let points = mapper.pack(&body);

        b.iter(|| encrypt(&mut rng, &schema, &public_key, &points));
    }

    #[bench]
    fn bench_decrypt_1024(b: &mut Bencher) {
        let body: Vec<u8> = (0..1024).map(|_| { rand::random::<u8>() }).collect();

        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let (private_key, public_key) = schema.generate_pair(&mut rng);
        let mapper = Mapper::new(256, &schema.curve);
        let points = mapper.pack(&body);
        let encrypted = encrypt(&mut rng, &schema, &public_key, &points);

        b.iter(|| decrypt(&schema, &private_key, &encrypted));
    }

    #[bench]
    fn bench_decrypt_10240(b: &mut Bencher) {
        let body: Vec<u8> = (0..10240).map(|_| { rand::random::<u8>() }).collect();

        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let (private_key, public_key) = schema.generate_pair(&mut rng);
        let mapper = Mapper::new(256, &schema.curve);
        let points = mapper.pack(&body);
        let encrypted = encrypt(&mut rng, &schema, &public_key, &points);

        b.iter(|| decrypt(&schema, &private_key, &encrypted));
    }
}
