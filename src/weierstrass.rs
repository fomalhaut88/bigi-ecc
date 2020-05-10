/*
Formula:
    y^2 = x^3 + A x + B
*/
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use bigi::prime::{add_mod, sub_mod, mul_mod, div_mod, sqrt_mod};
use crate::{point, point_zero};
use crate::base::{Point, CurveTrait};


#[derive(Copy, Clone)]
pub struct WeierstrassCurve {
    pub a: Bigi,
    pub b: Bigi,
    pub m: Bigi
}


impl WeierstrassCurve {
    fn left(&self, y: &Bigi) -> Bigi {
        mul_mod(&y, &y, &self.m)
    }

    fn right(&self, x: &Bigi) -> Bigi {
        add_mod(
            &mul_mod(
                &add_mod(
                    &mul_mod(&x, &x, &self.m),
                    &self.a, &self.m
                ), &x, &self.m
            ), &self.b, &self.m
        )
    }
}


impl CurveTrait for WeierstrassCurve {
    fn get_modulo(&self) -> Bigi {
        self.m
    }

    fn zero(&self) -> Point {
        point_zero!()
    }

    fn check(&self, p: &Point) -> bool {
        if p.is_zero {
            true
        } else {
            self.left(&p.y) == self.right(&p.x)
        }
    }

    fn find_y(&self, x: &Bigi) -> Result<(Bigi, Bigi), &'static str> {
        let y2 = self.right(&x);
        let roots = sqrt_mod(&y2, &self.m)?;
        Ok(roots)
    }

    fn inv(&self, p: &Point) -> Point {
        point!(p.x, self.m - &p.y)
    }

    fn add(&self, p: &Point, q: &Point) -> Point {
        if q.is_zero {
            return *p;
        }
        if p.is_zero {
            return *q;
        }
        if (p.x == q.x) && ((p.y != q.y) || p.y.is_zero()) {
            return point_zero!();
        }

        let alpha = {
            if p.x == q.x {
                // alpha = (3 * x^2 + a) / (2y)
                div_mod(
                    &add_mod(
                        &mul_mod(
                            &mul_mod(&p.x, &p.x, &self.m),
                            &bigi![3], &self.m
                        ), &self.a, &self.m
                    ),
                    &mul_mod(&p.y, &bigi![2], &self.m),
                    &self.m
                )
            } else {
                // alpha = (Py - Qy) / (Px - Qx)
                div_mod(
                    &sub_mod(&p.y, &q.y, &self.m),
                    &sub_mod(&p.x, &q.x, &self.m),
                    &self.m
                )
            }
        };

        // Rx := alpha^2 - (Px + Qx)
        let x = sub_mod(
            &mul_mod(&alpha, &alpha, &self.m),
            &add_mod(&p.x, &q.x, &self.m),
            &self.m
        );

        // Ry := (Qx - Rx) * alpha - Qy
        let y = sub_mod(
            &mul_mod(
                &sub_mod(&q.x, &x, &self.m),
                &alpha, &self.m
            ), &q.y, &self.m
        );

        point!(x, y)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::point_simple;
    use crate::schemas::{load_secp256k1, load_fp254bnb};
    use test::Bencher;

    #[test]
    fn test_check() {
        let curve = WeierstrassCurve {
            a: bigi![2],
            b: bigi![3],
            m: bigi![97]
        };
        assert_eq!(curve.check(&point_simple!(80, 87)), true);
        assert_eq!(curve.check(&point_simple!(0, 0)), false);
        assert_eq!(curve.check(&point_zero!()), true);
        assert_eq!(curve.check(&point_simple!(80, 86)), false);
        assert_eq!(curve.check(&point_simple!(30, 0)), true);
    }

    #[test]
    fn test_add() {
        let curve = WeierstrassCurve {
            a: bigi![2],
            b: bigi![3],
            m: bigi![97]
        };

        assert_eq!(curve.add(&point_simple!(3, 6), &point_simple!(80, 10)), point_simple!(80, 87));
        assert_eq!(curve.add(&point_simple!(3, 6), &point_zero!()), point_simple!(3, 6));
        assert_eq!(curve.add(&point_zero!(), &point_simple!(3, 6)), point_simple!(3, 6));
        assert_eq!(curve.add(&point_zero!(), &point_zero!()), point_zero!());
        assert_eq!(curve.add(&point_simple!(3, 6), &point_simple!(3, 91)), point_zero!());
        assert_eq!(curve.add(&point_simple!(30, 0), &point_simple!(68, 0)), point_simple!(96, 0));
    }

    #[test]
    fn test_double() {
        let curve = WeierstrassCurve {
            a: bigi![2],
            b: bigi![3],
            m: bigi![97]
        };

        assert_eq!(curve.double(&point_simple!(3, 6)), point_simple!(80, 10));
        assert_eq!(curve.double(&point_zero!()), point_zero!());
        assert_eq!(curve.double(&point_simple!(30, 0)), point_zero!());
    }

    #[test]
    fn test_mul() {
        let curve = WeierstrassCurve {
            a: bigi![2],
            b: bigi![3],
            m: bigi![97]
        };

        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![0]), point_zero!());
        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![1]), point_simple!(3, 6));
        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![2]), point_simple!(80, 10));
        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![3]), point_simple!(80, 87));
        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![4]), point_simple!(3, 91));
        assert_eq!(curve.mul(&point_simple!(3, 6), &bigi![5]), point_zero!());
    }

    #[test]
    fn test_secp256k1() {
        let schema = load_secp256k1();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(&schema.get_point(&bigi![25])), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[test]
    fn test_fp254bnb() {
        let schema = load_fp254bnb();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(&schema.get_point(&bigi![25])), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[bench]
    fn bench_secp256k1_generate_pair(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        b.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_secp256k1_add(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k1 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        b.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_secp256k1_double(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_secp256k1_mul(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_secp256k1_check(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_secp256k1_inv(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_secp256k1_find_y(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.find_y(&p.x));
    }

    #[bench]
    fn bench_fp254bnb_generate_pair(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        b.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_fp254bnb_add(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k1 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        b.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_fp254bnb_double(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_fp254bnb_mul(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_fp254bnb_check(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_fp254bnb_inv(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_fp254bnb_find_y(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.find_y(&p.x));
    }
}
