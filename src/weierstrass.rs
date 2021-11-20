//! This module implements [Weierstrass curve](https://en.wikipedia.org/wiki/Elliptic_curve)
//! that is defined by the equation `y^2 = x^3 + A x + B`.
use bigi::Bigi;
use bigi::prime::{add_mod, sub_mod, mul_mod, div_mod, sqrt_mod};
use crate::{point, point_zero};
use crate::base::{Point, CurveTrait};


/// Weierstrass curve type.
#[derive(Copy, Clone)]
pub struct WeierstrassCurve<const N: usize> {
    pub a: Bigi<N>,
    pub b: Bigi<N>,
    pub m: Bigi<N>
}


impl<const N: usize> WeierstrassCurve<N> {
    fn left(&self, y: &Bigi<N>) -> Bigi<N> {
        mul_mod(&y, &y, &self.m)
    }

    fn right(&self, x: &Bigi<N>) -> Bigi<N> {
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


impl<const N: usize> CurveTrait<N> for WeierstrassCurve<N> {
    fn get_modulo(&self) -> Bigi<N> {
        self.m
    }

    fn zero(&self) -> Point<N> {
        point_zero!(N)
    }

    fn check(&self, p: &Point<N>) -> bool {
        if p.is_zero {
            true
        } else {
            self.left(&p.y) == self.right(&p.x)
        }
    }

    fn find_y(&self, x: &Bigi<N>) -> Result<(Bigi<N>, Bigi<N>), &'static str> {
        let y2 = self.right(&x);
        let roots = sqrt_mod(&y2, &self.m)?;
        Ok(roots)
    }

    fn inv(&self, p: &Point<N>) -> Point<N> {
        point!(p.x, self.m - &p.y)
    }

    fn add(&self, p: &Point<N>, q: &Point<N>) -> Point<N> {
        if q.is_zero {
            return *p;
        }
        if p.is_zero {
            return *q;
        }
        if (p.x == q.x) && ((p.y != q.y) || p.y.is_zero()) {
            return point_zero!(N);
        }

        let alpha = {
            if p.x == q.x {
                // alpha = (3 * x^2 + a) / (2y)
                div_mod(
                    &add_mod(
                        &mul_mod(
                            &mul_mod(&p.x, &p.x, &self.m),
                            &Bigi::<N>::from(3), &self.m
                        ), &self.a, &self.m
                    ),
                    &mul_mod(&p.y, &Bigi::<N>::from(2), &self.m),
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
    use bigi::bigi;
    use crate::point_simple;
    use crate::schemas::{load_secp256k1, load_fp254bnb};
    use test::Bencher;

    #[test]
    fn test_check() {
        let curve = WeierstrassCurve {
            a: bigi![8; 2],
            b: bigi![8; 3],
            m: bigi![8; 97]
        };
        assert_eq!(curve.check(&point_simple!(8; 80, 87)), true);
        assert_eq!(curve.check(&point_simple!(8; 0, 0)), false);
        assert_eq!(curve.check(&point_zero!(8)), true);
        assert_eq!(curve.check(&point_simple!(8; 80, 86)), false);
        assert_eq!(curve.check(&point_simple!(8; 30, 0)), true);
    }

    #[test]
    fn test_add() {
        let curve = WeierstrassCurve {
            a: bigi![8; 2],
            b: bigi![8; 3],
            m: bigi![8; 97]
        };

        assert_eq!(
            curve.add(&point_simple!(8; 3, 6), &point_simple!(8; 80, 10)),
            point_simple!(8; 80, 87)
        );
        assert_eq!(
            curve.add(&point_simple!(8; 3, 6), &point_zero!(8)),
            point_simple!(8; 3, 6)
        );
        assert_eq!(
            curve.add(&point_zero!(8), &point_simple!(8; 3, 6)),
            point_simple!(8; 3, 6)
        );
        assert_eq!(
            curve.add(&point_zero!(8), &point_zero!(8)),
            point_zero!(8)
        );
        assert_eq!(
            curve.add(&point_simple!(8; 3, 6), &point_simple!(8; 3, 91)),
            point_zero!(8)
        );
        assert_eq!(
            curve.add(&point_simple!(8; 30, 0), &point_simple!(8; 68, 0)),
            point_simple!(8; 96, 0)
        );
    }

    #[test]
    fn test_double() {
        let curve = WeierstrassCurve {
            a: bigi![8; 2],
            b: bigi![8; 3],
            m: bigi![8; 97]
        };

        assert_eq!(curve.double(
            &point_simple!(8; 3, 6)), point_simple!(8; 80, 10));
        assert_eq!(curve.double(&point_zero!(8)), point_zero!(8));
        assert_eq!(curve.double(&point_simple!(8; 30, 0)), point_zero!(8));
    }

    #[test]
    fn test_mul() {
        let curve = WeierstrassCurve {
            a: bigi![8; 2],
            b: bigi![8; 3],
            m: bigi![8; 97]
        };

        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 0]), point_zero!(8));
        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 1]), point_simple!(8; 3, 6));
        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 2]), point_simple!(8; 80, 10));
        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 3]), point_simple!(8; 80, 87));
        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 4]), point_simple!(8; 3, 91));
        assert_eq!(curve.mul(
            &point_simple!(8; 3, 6), &bigi![8; 5]), point_zero!(8));
    }

    #[test]
    fn test_secp256k1() {
        let schema = load_secp256k1();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(
            &schema.get_point(&Bigi::<8>::from(25))), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[test]
    fn test_fp254bnb() {
        let schema = load_fp254bnb();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(
            &schema.get_point(&Bigi::<8>::from(25))), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[bench]
    fn bench_secp256k1_generate_pair(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        bencher.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_secp256k1_add(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k1 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        bencher.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_secp256k1_double(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_secp256k1_mul(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_secp256k1_check(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_secp256k1_inv(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_secp256k1_find_y(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_secp256k1();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.find_y(&p.x));
    }

    #[bench]
    fn bench_fp254bnb_generate_pair(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        bencher.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_fp254bnb_add(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k1 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        bencher.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_fp254bnb_double(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_fp254bnb_mul(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_fp254bnb_check(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_fp254bnb_inv(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_fp254bnb_find_y(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_fp254bnb();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.find_y(&p.x));
    }
}
