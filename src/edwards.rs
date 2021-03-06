/*
Formula:
    x^2 + y^2 = 1 + D x^2 y^2
*/
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use bigi::prime::{add_mod, sub_mod, mul_mod, div_mod, sqrt_mod};
use crate::{point};
use crate::base::{Point, CurveTrait};


#[derive(Copy, Clone)]
pub struct EdwardsCurve {
    pub d: Bigi,
    pub m: Bigi
}


impl CurveTrait for EdwardsCurve {
    fn get_modulo(&self) -> Bigi {
        self.m
    }

    fn zero(&self) -> Point {
        point!(bigi![0], bigi![1])
    }

    fn check(&self, p: &Point) -> bool {
        let left = add_mod(
            &mul_mod(&p.x, &p.x, &self.m),
            &mul_mod(&p.y, &p.y, &self.m),
            &self.m
        );
        let right = add_mod(
            &mul_mod(
                &mul_mod(
                    &mul_mod(&p.x, &p.x, &self.m),
                    &mul_mod(&p.y, &p.y, &self.m),
                    &self.m
                ),
                &self.d, &self.m
            ),
            &bigi![1], &self.m
        );
        left == right
    }

    fn find_y(&self, x: &Bigi) -> Result<(Bigi, Bigi), &'static str> {
        let a = sub_mod(
            &mul_mod(&x, &x, &self.m),
            &bigi![1], &self.m
        );
        let b = sub_mod(
            &mul_mod(
                &mul_mod(&x, &x, &self.m),
                &self.d, &self.m
            ),
            &bigi![1], &self.m
        );
        let y2 = div_mod(&a, &b, &self.m);
        let roots = sqrt_mod(&y2, &self.m)?;
        Ok(roots)
    }

    fn inv(&self, p: &Point) -> Point {
        point!(self.m - &p.x, p.y)
    }

    fn add(&self, p: &Point, q: &Point) -> Point {
        // t := D Px Qx Py Qy
        let t = mul_mod(
            &mul_mod(
                &mul_mod(&p.x, &q.x, &self.m),
                &mul_mod(&p.y, &q.y, &self.m),
                &self.m
            ),
            &self.d, &self.m
        );
        // x := (Px Qy + Py Qx) / (1 + t)
        let x = div_mod(
            &add_mod(
                &mul_mod(&p.x, &q.y, &self.m),
                &mul_mod(&q.x, &p.y, &self.m),
                &self.m
            ),
            &add_mod(&bigi![1], &t, &self.m),
            &self.m
        );
        // y := (Py Qy - Px Qx) / (1 - t)
        let y = div_mod(
            &sub_mod(
                &mul_mod(&p.y, &q.y, &self.m),
                &mul_mod(&p.x, &q.x, &self.m),
                &self.m
            ),
            &sub_mod(&bigi![1], &t, &self.m),
            &self.m
        );
        point!(x, y)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::point_simple;
    use crate::schemas::load_curve1174;
    use test::Bencher;

    #[test]
    fn test_check() {
        let curve = EdwardsCurve {
            d: bigi![2],
            m: bigi![97]
        };
        assert_eq!(curve.check(&point_simple!(48, 27)), true);
        assert_eq!(curve.check(&point_simple!(0, 0)), false);
        assert_eq!(curve.check(&curve.zero()), true);
        assert_eq!(curve.check(&point_simple!(48, 28)), false);
    }

    #[test]
    fn test_add() {
        let curve = EdwardsCurve {
            d: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.add(&point_simple!(5, 40), &point_simple!(48, 27)), point_simple!(27, 48));
        assert_eq!(curve.add(&point_simple!(5, 40), &curve.zero()), point_simple!(5, 40));
        assert_eq!(curve.add(&curve.zero(), &point_simple!(5, 40)), point_simple!(5, 40));
        assert_eq!(curve.add(&curve.zero(), &curve.zero()), curve.zero());
        assert_eq!(curve.add(&point_simple!(5, 40), &point_simple!(92, 40)), curve.zero());
    }

    #[test]
    fn test_double() {
        let curve = EdwardsCurve {
            d: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.double(&point_simple!(5, 40)), point_simple!(48, 27));
        assert_eq!(curve.double(&curve.zero()), curve.zero());
        assert_eq!(curve.double(&point_simple!(0, 96)), curve.zero());
    }

    #[test]
    fn test_mul() {
        let curve = EdwardsCurve {
            d: bigi![2],
            m: bigi![97]
        };

        assert_eq!(curve.mul(&point_simple!(5, 40), &bigi![0]), curve.zero());
        assert_eq!(curve.mul(&point_simple!(5, 40), &bigi![1]), point_simple!(5, 40));
        assert_eq!(curve.mul(&point_simple!(5, 40), &bigi![2]), point_simple!(48, 27));
        assert_eq!(curve.mul(&point_simple!(5, 40), &bigi![3]), point_simple!(27, 48));
        assert_eq!(curve.mul(&point_simple!(5, 40), &bigi![20]), curve.zero());
    }

    #[test]
    fn test_curve1174() {
        let schema = load_curve1174();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(&schema.get_point(&bigi![25])), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[bench]
    fn bench_curve1174_generate_pair(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        b.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_curve1174_add(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k1 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        b.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_curve1174_double(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_curve1174_mul(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_curve1174_check(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_curve1174_inv(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_curve1174_find_y(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::gen_random(&mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        b.iter(|| schema.curve.find_y(&p.x));
    }
}
