use std::fmt;
use bigi::{Bigi, BIGI_TYPE_BITS, BIGI_BYTES};


#[derive(Copy, Clone)]
pub struct Point {
    pub x: Bigi,
    pub y: Bigi,
    pub is_zero: bool
}


#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y, is_zero: false }
    }
}


#[macro_export]
macro_rules! point_zero {
    () => {
        Point { x: bigi![0], y: bigi![0], is_zero: true }
    }
}


#[macro_export]
macro_rules! point_simple {
    ($x:expr, $y:expr) => {
        Point { x: bigi![$x], y: bigi![$y], is_zero: false }
    }
}


impl PartialEq for Point {
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


impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_zero {
            write!(f, "{{null}}")
        } else {
            write!(f, "{{{}, {}}}", self.x, self.y)
        }
    }
}


impl Point {
    pub fn to_hex(&self) -> String {
        format!("{} {}", self.x.to_hex(), self.y.to_hex())
    }

    pub fn from_hex(hex: &str) -> Point {
        let v: Vec<&str> = hex.split_whitespace().collect();
        point!(Bigi::from_hex(v[0]), Bigi::from_hex(v[1]))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(&self.x.to_bytes());
        res.extend(&self.y.to_bytes());
        res
    }

    pub fn from_bytes(bytes: &[u8]) -> Point {
        point!(
            Bigi::from_bytes(&bytes[0..BIGI_BYTES]),
            Bigi::from_bytes(&bytes[BIGI_BYTES..(2 * BIGI_BYTES)])
        )
    }
}


pub trait CurveTrait {
    fn get_modulo(&self) -> Bigi;
    fn zero(&self) -> Point;
    fn check(&self, p: &Point) -> bool;
    fn find_y(&self, x: &Bigi) -> Result<(Bigi, Bigi), &'static str>;
    fn inv(&self, p: &Point) -> Point;
    fn add(&self, p: &Point, q: &Point) -> Point;

    fn double(&self, p: &Point) -> Point {
        self.add(&p, &p)
    }

    fn mul(&self, p: &Point, k: &Bigi) -> Point {
        let mut res = self.zero();
        let mut p2 = p.clone();
        for i in 0..(BIGI_TYPE_BITS * k.order) {
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
    use bigi::{bigi, BIGI_MAX_DIGITS};

    #[test]
    fn test_to_hex() {
        assert_eq!(point_simple!(1234, 1255).to_hex(), "0x4D2 0x4E7");
    }

    #[test]
    fn test_from_hex() {
        assert_eq!(Point::from_hex("0x4D2 0x4E7"), point_simple!(1234, 1255));
    }

    #[test]
    fn test_to_bytes() {
        assert_eq!(
            point_simple!(2, 3).to_bytes(),
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
            Point::from_bytes(
                &vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            ), point_simple!(2, 3)
        );
    }
}
