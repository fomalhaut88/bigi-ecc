use std::fmt;
use bigi::{Bigi, BIGI_BYTES};


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
    use bigi::{bigi, BIGI_MAX_DIGITS};
    use test::Bencher;

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

    #[bench]
    fn bench_to_hex_256(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let x = Bigi::gen_random(&mut rng, 256, false);
        let y = Bigi::gen_random(&mut rng, 256, false);
        let p = point!(x, y);
        b.iter(|| p.to_hex());
    }

    #[bench]
    fn bench_to_bytes_256(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let x = Bigi::gen_random(&mut rng, 256, false);
        let y = Bigi::gen_random(&mut rng, 256, false);
        let p = point!(x, y);
        b.iter(|| p.to_bytes());
    }

    #[bench]
    fn bench_from_hex_256(b: &mut Bencher) {
        b.iter(|| Point::from_hex("0xCA0D6AB29A21576F099A19B90DE2AFAF0A350DA6CA630725130E3F6A2BE77EB2 0xAE8B83698251862BE2C4D808149099D9D6525EF141B10C90BFF745094D1C9861"));
    }

    #[bench]
    fn bench_from_bytes_256(b: &mut Bencher) {
        b.iter(|| Point::from_bytes(
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
