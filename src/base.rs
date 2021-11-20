//! This module implements basics types of the library like point on a curve
//! and the curve trait.

use std::{fmt, mem};
use bigi::Bigi;


/// Generic type for point on a curve that is a pair of two integers.
#[derive(Copy, Clone)]
pub struct Point<const N: usize> {
    pub x: Bigi<N>,
    pub y: Bigi<N>,
    pub is_zero: bool
}


/// A macros to define a point.
/// ```rust
/// use bigi::Bigi;
/// use bigi_ecc::{Point, point};
///
/// let p = point!(Bigi::<8>::from(5), Bigi::<8>::from(3));
/// ```
#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y, is_zero: false }
    }
}


/// A macros to define a zero point.
/// ```rust
/// use bigi::Bigi;
/// use bigi_ecc::{Point, point_zero};
///
/// let p = point_zero!(8);
/// ```
#[macro_export]
macro_rules! point_zero {
    ($n:expr) => {
        Point {
            x: Bigi::<$n>::from(0),
            y: Bigi::<$n>::from(0),
            is_zero: true
        }
    }
}


/// A macros to define a poing from `u64` integers.
/// ```rust
/// use bigi::Bigi;
/// use bigi_ecc::{Point, point_simple};
///
/// let p = point_simple!(8; 5, 3);
/// ```
#[macro_export]
macro_rules! point_simple {
    ($n:expr; $x:expr, $y:expr) => {
        Point {
            x: Bigi::<$n>::from($x),
            y: Bigi::<$n>::from($y),
            is_zero: false
        }
    }
}


impl<const N: usize> PartialEq for Point<N> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_zero == other.is_zero {
            if self.is_zero {
                true
            } else {
                (self.x == other.x) && (self.y == other.y)
            }
        } else {
            false
        }
    }
}


impl<const N: usize> fmt::Debug for Point<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_zero {
            write!(f, "{{null}}")
        } else {
            write!(f, "{{{:?}, {:?}}}", self.x, self.y)
        }
    }
}


impl<const N: usize> Point<N> {
    /// Converts point to a hex string.
    pub fn to_hex(&self) -> String {
        format!("{} {}", self.x.to_hex(), self.y.to_hex())
    }

    /// Creates point from a hex string.
    pub fn from_hex(hex: &str) -> Point<N> {
        let v: Vec<&str> = hex.split_whitespace().collect();
        point!(Bigi::from_hex(v[0]), Bigi::from_hex(v[1]))
    }

    /// Converts point to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(&self.x.to_bytes());
        res.extend(&self.y.to_bytes());
        res
    }

    /// Creates point from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Point<N> {
        let bigi_size = mem::size_of::<Bigi<N>>();
        point!(
            Bigi::from_bytes(&bytes[0..bigi_size]),
            Bigi::from_bytes(&bytes[bigi_size..(bigi_size << 1)])
        )
    }
}


/// `CurveTrait` is a trait that defines all the necessary methods
/// to make the algorithms in the library work.
pub trait CurveTrait<const N: usize> {
    /// Gets modulo of the curve.
    fn get_modulo(&self) -> Bigi<N>;

    /// Gets zero point of the curve.
    fn zero(&self) -> Point<N>;

    /// Returns true if the point is on the curve else false.
    fn check(&self, p: &Point<N>) -> bool;

    /// Finds `y` coordinates of two points on the curve by given `x`.
    fn find_y(&self, x: &Bigi<N>) -> Result<(Bigi<N>, Bigi<N>), &'static str>;

    /// Gets the inverse of the point.
    fn inv(&self, p: &Point<N>) -> Point<N>;

    /// Sum of the points on the curve.
    fn add(&self, p: &Point<N>, q: &Point<N>) -> Point<N>;

    /// Doubles the point on the curve.
    fn double(&self, p: &Point<N>) -> Point<N> {
        self.add(&p, &p)
    }

    /// Multiplies the point by the integer.
    fn mul(&self, p: &Point<N>, k: &Bigi<N>) -> Point<N> {
        let mut res = self.zero();
        let mut p2 = p.clone();
        for i in 0..k.bit_length() {
            if k.get_bit(i) {
                res = self.add(&res, &p2);
            }
            p2 = self.double(&p2);
        }
        res
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_to_hex() {
        assert_eq!(point_simple!(8; 1234, 1255).to_hex(), "0x4D2 0x4E7");
    }

    #[test]
    fn test_from_hex() {
        assert_eq!(Point::<8>::from_hex("0x4D2 0x4E7"),
                   point_simple!(8; 1234, 1255));
    }

    #[test]
    fn test_to_bytes() {
        assert_eq!(
            point_simple!(8; 2, 3).to_bytes(),
            vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Point::<8>::from_bytes(
                &vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            ), point_simple!(8; 2, 3)
        );
    }

    #[bench]
    fn bench_to_hex_256(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let x = Bigi::<8>::gen_random(&mut rng, 256, false);
        let y = Bigi::<8>::gen_random(&mut rng, 256, false);
        let p = point!(x, y);
        b.iter(|| p.to_hex());
    }

    #[bench]
    fn bench_to_bytes_256(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let x = Bigi::<8>::gen_random(&mut rng, 256, false);
        let y = Bigi::<8>::gen_random(&mut rng, 256, false);
        let p = point!(x, y);
        b.iter(|| p.to_bytes());
    }

    #[bench]
    fn bench_from_hex_256(b: &mut Bencher) {
        b.iter(|| Point::<8>::from_hex("0xCA0D6AB29A21576F099A19B90DE2AFAF0A350DA6CA630725130E3F6A2BE77EB2 0xAE8B83698251862BE2C4D808149099D9D6525EF141B10C90BFF745094D1C9861"));
    }

    #[bench]
    fn bench_from_bytes_256(b: &mut Bencher) {
        b.iter(|| Point::<8>::from_bytes(
            &vec![158, 17, 67, 29, 164, 141, 187, 19, 131, 247, 203, 136, 74,
                  245, 44, 55, 52, 199, 147, 42, 68, 57, 169, 223, 210, 162,
                  60, 196, 45, 7, 59, 77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                  164, 76, 176, 189, 26, 65, 21, 46, 51, 209, 243, 12, 68, 246,
                  75, 198, 43, 40, 139, 106, 66, 13, 3, 66, 37, 197, 108, 27,
                  52, 222, 83, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ));
    }
}
