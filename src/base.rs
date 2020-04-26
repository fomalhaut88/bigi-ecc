use std::fmt;
use bigi::{Bigi, BIGI_TYPE_BITS};


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
