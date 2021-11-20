//! This module implements [Edwards curve](https://en.wikipedia.org/wiki/Edwards_curve)
//! that is defined by the equation `x^2 + y^2 = 1 + D x^2 y^2`.
use bigi::Bigi;
use bigi::prime::{add_mod, sub_mod, mul_mod, div_mod, sqrt_mod};
use crate::{point};
use crate::base::{Point, CurveTrait};


/// Edwards curve type.
#[derive(Copy, Clone)]
pub struct EdwardsCurve<const N: usize> {
    pub d: Bigi<N>,
    pub m: Bigi<N>
}


impl<const N: usize> CurveTrait<N> for EdwardsCurve<N> {
    fn get_modulo(&self) -> Bigi<N> {
        self.m
    }

    fn zero(&self) -> Point<N> {
        point!(Bigi::<N>::from(0), Bigi::<N>::from(1))
    }

    fn check(&self, p: &Point<N>) -> bool {
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
            &Bigi::<N>::from(1), &self.m
        );
        left == right
    }

    fn find_y(&self, x: &Bigi<N>) -> Result<(Bigi<N>, Bigi<N>), &'static str> {
        let a = sub_mod(
            &mul_mod(&x, &x, &self.m),
            &Bigi::<N>::from(1), &self.m
        );
        let b = sub_mod(
            &mul_mod(
                &mul_mod(&x, &x, &self.m),
                &self.d, &self.m
            ),
            &Bigi::<N>::from(1), &self.m
        );
        let y2 = div_mod(&a, &b, &self.m);
        let roots = sqrt_mod(&y2, &self.m)?;
        Ok(roots)
    }

    fn inv(&self, p: &Point<N>) -> Point<N> {
        point!(self.m - &p.x, p.y)
    }

    fn add(&self, p: &Point<N>, q: &Point<N>) -> Point<N> {
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
            &add_mod(&Bigi::<N>::from(1), &t, &self.m),
            &self.m
        );
        // y := (Py Qy - Px Qx) / (1 - t)
        let y = div_mod(
            &sub_mod(
                &mul_mod(&p.y, &q.y, &self.m),
                &mul_mod(&p.x, &q.x, &self.m),
                &self.m
            ),
            &sub_mod(&Bigi::<N>::from(1), &t, &self.m),
            &self.m
        );
        point!(x, y)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bigi::bigi;
    use crate::point_simple;
    use crate::schemas::load_curve1174;
    use test::Bencher;

    #[test]
    fn test_check() {
        let curve = EdwardsCurve {
            d: bigi![8; 2],
            m: bigi![8; 97]
        };
        assert_eq!(curve.check(&point_simple!(8; 48, 27)), true);
        assert_eq!(curve.check(&point_simple!(8; 0, 0)), false);
        assert_eq!(curve.check(&curve.zero()), true);
        assert_eq!(curve.check(&point_simple!(8; 48, 28)), false);
    }

    #[test]
    fn test_add() {
        let curve = EdwardsCurve {
            d: bigi![8; 2],
            m: bigi![8; 97]
        };

        assert_eq!(curve.add(&point_simple!(8; 5, 40), &point_simple!(8; 48, 27)), point_simple!(8; 27, 48));
        assert_eq!(curve.add(&point_simple!(8; 5, 40), &curve.zero()), point_simple!(8; 5, 40));
        assert_eq!(curve.add(&curve.zero(), &point_simple!(8; 5, 40)), point_simple!(8; 5, 40));
        assert_eq!(curve.add(&curve.zero(), &curve.zero()), curve.zero());
        assert_eq!(curve.add(&point_simple!(8; 5, 40), &point_simple!(8; 92, 40)), curve.zero());
    }

    #[test]
    fn test_double() {
        let curve = EdwardsCurve {
            d: bigi![8; 2],
            m: bigi![8; 97]
        };

        assert_eq!(curve.double(&point_simple!(8; 5, 40)), point_simple!(8; 48, 27));
        assert_eq!(curve.double(&curve.zero()), curve.zero());
        assert_eq!(curve.double(&point_simple!(8; 0, 96)), curve.zero());
    }

    #[test]
    fn test_mul() {
        let curve = EdwardsCurve {
            d: bigi![8; 2],
            m: bigi![8; 97]
        };

        assert_eq!(curve.mul(&point_simple!(8; 5, 40), &bigi![8; 0]), curve.zero());
        assert_eq!(curve.mul(&point_simple!(8; 5, 40), &bigi![8; 1]), point_simple!(8; 5, 40));
        assert_eq!(curve.mul(&point_simple!(8; 5, 40), &bigi![8; 2]), point_simple!(8; 48, 27));
        assert_eq!(curve.mul(&point_simple!(8; 5, 40), &bigi![8; 3]), point_simple!(8; 27, 48));
        assert_eq!(curve.mul(&point_simple!(8; 5, 40), &bigi![8; 20]), curve.zero());
    }

    #[test]
    fn test_curve1174() {
        let schema = load_curve1174();
        assert_eq!(schema.curve.check(&schema.generator), true);
        assert_eq!(schema.curve.check(&schema.get_point(&bigi![8; 25])), true);
        assert_eq!(schema.get_point(&schema.order), schema.curve.zero());
    }

    #[bench]
    fn bench_curve1174_generate_pair(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        bencher.iter(|| schema.generate_pair(&mut rng));
    }

    #[bench]
    fn bench_curve1174_add(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k1 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let k2 = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p1 = schema.get_point(&k1);
        let p2 = schema.get_point(&k2);
        bencher.iter(|| schema.curve.add(&p1, &p2));
    }

    #[bench]
    fn bench_curve1174_double(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.double(&p));
    }

    #[bench]
    fn bench_curve1174_mul(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let l = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.mul(&p, &l));
    }

    #[bench]
    fn bench_curve1174_check(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.check(&p));
    }

    #[bench]
    fn bench_curve1174_inv(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.inv(&p));
    }

    #[bench]
    fn bench_curve1174_find_y(bencher: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let schema = load_curve1174();
        let k = Bigi::<8>::gen_random(
            &mut rng, schema.bits, false) % &schema.order;
        let p = schema.get_point(&k);
        bencher.iter(|| schema.curve.find_y(&p.x));
    }
}
