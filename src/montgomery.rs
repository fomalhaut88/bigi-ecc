/*
Formula:
    B y^2 = x^3 + A x^2 + x
*/
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use bigi::prime::{add_mod, sub_mod, mul_mod, div_mod, sqrt_mod};
use crate::{point, point_zero};
use crate::base::{Point, CurveTrait};


#[derive(Copy, Clone)]
pub struct MontgomeryCurve {
    pub a: Bigi,
    pub b: Bigi,
    pub m: Bigi
}


impl MontgomeryCurve {
    fn left(&self, y: &Bigi) -> Bigi {
        mul_mod(
            &mul_mod(&y, &y, &self.m),
            &self.b, &self.m
        )
    }

    fn right(&self, x: &Bigi) -> Bigi {
        mul_mod(
            &add_mod(
                &mul_mod(
                    &add_mod(&x, &self.a, &self.m),
                    &x, &self.m
                ),
                &bigi![1], &self.m
            ),
            &x, &self.m
        )
    }
}


impl CurveTrait for MontgomeryCurve {
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
        let right = self.right(&x);
        let y2 = div_mod(&right, &self.b, &self.m);
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
        if (p.x == q.x) && ((p.y != q.y) || (p.y == bigi![0])) {
            return point_zero!();
        }

        let alpha = {
            if p.x == q.x {
                // alpha = ((3 x + 2 a) x + 1) / (2 B y)
                div_mod(
                    &add_mod(
                        &mul_mod(
                            &add_mod(
                                &mul_mod(&p.x, &bigi![3], &self.m),
                                &mul_mod(&self.a, &bigi![2], &self.m),
                                &self.m
                            ),
                            &p.x, &self.m
                        ), &bigi![1], &self.m
                    ),
                    &mul_mod(
                        &mul_mod(&p.y, &bigi![2], &self.m),
                        &self.b, &self.m
                    ),
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

        // Rx := B alpha^2 - (Px + Qx + A)
        let x = sub_mod(
            &mul_mod(
                &mul_mod(&alpha, &alpha, &self.m),
                &self.b, &self.m
            ),
            &add_mod(
                &add_mod(&p.x, &q.x, &self.m),
                &self.a, &self.m
            ),
            &self.m
        );

        // Ry := (Qx - Rx) * alpha - Qy
        let y = sub_mod(
            &mul_mod(
                &sub_mod(&q.x, &x, &self.m
                ), &alpha, &self.m
            ), &q.y, &self.m
        );

        point!(x, y)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::point_simple;
    use crate::schemas::load_curve25519;
    use test::Bencher;

    #[test]
    fn test_check() {
        let curve = MontgomeryCurve {
            a: bigi![5],
            b: bigi![2],
            m: bigi![97]
        };
        assert_eq!(curve.check(&point_simple!(65, 15)), true);
        assert_eq!(curve.check(&point_simple!(0, 0)), true);
        assert_eq!(curve.check(&point_zero!()), true);
        assert_eq!(curve.check(&point_simple!(65, 81)), false);
    }

    #[test]
    fn test_add() {
        let curve = MontgomeryCurve {
            a: bigi![5],
            b: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.add(&point_simple!(12, 39), &point_simple!(65, 15)), point_simple!(18, 90));
        assert_eq!(curve.add(&point_simple!(12, 39), &point_zero!()), point_simple!(12, 39));
        assert_eq!(curve.add(&point_zero!(), &point_simple!(12, 39)), point_simple!(12, 39));
        assert_eq!(curve.add(&point_zero!(), &point_zero!()), point_zero!());
        assert_eq!(curve.add(&point_simple!(12, 39), &point_simple!(12, 58)), point_zero!());
    }

    #[test]
    fn test_double() {
        let curve = MontgomeryCurve {
            a: bigi![5],
            b: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.double(&point_simple!(12, 39)), point_simple!(65, 15));
        assert_eq!(curve.double(&point_zero!()), point_zero!());
        assert_eq!(curve.double(&point_simple!(0, 0)), point_zero!());
    }

    #[test]
    fn test_mul() {
        let curve = MontgomeryCurve {
            a: bigi![5],
            b: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.mul(&point_simple!(12, 39), &bigi![0]), point_zero!());
        assert_eq!(curve.mul(&point_simple!(12, 39), &bigi![1]), point_simple!(12, 39));
        assert_eq!(curve.mul(&point_simple!(12, 39), &bigi![2]), point_simple!(65, 15));
        assert_eq!(curve.mul(&point_simple!(12, 39), &bigi![3]), point_simple!(18, 90));
        assert_eq!(curve.mul(&point_simple!(12, 39), &bigi![11]), point_zero!());
    }

    #[test]
    fn test_curve25519() {
        let schema = load_curve25519();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(&schema.get_point(&bigi![25])), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[bench]
    fn bench_curve25519_generate_pair(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        b.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_curve25519_add(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k1 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        b.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_curve25519_double(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_curve25519_mul(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_curve25519_check(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_curve25519_inv(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_curve25519_find_y(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve25519();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.find_y(&p.x));
    }
}
