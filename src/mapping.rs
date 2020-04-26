use std::cmp;
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use crate::{point};
use crate::base::{Point, CurveTrait};


pub struct Mapper {
    block_size: usize
}


impl Mapper {
    pub fn new(bits: usize) -> Self {
        Self {
            block_size: bits / 8 - 1
        }
    }

    pub fn pack<T: CurveTrait>(&self, body: &Vec<u8>, curve: &T) -> Vec<Point> {
        let size = body.len();
        let mut points: Vec<Point> = Vec::new();

        for i in (0..size).step_by(self.block_size) {
            let end = cmp::min(i + self.block_size, size);

            let block = &body[i..end];
            let (x, y) = {
                let res;
                let mut x = Bigi::from_bytes(&block).unwrap() << 8;
                loop {
                    match curve.find_y(&x) {
                        Ok(roots) => { res = (x, roots.0); break; },
                        Err(_e) => { x += &bigi![1] }
                    }
                }
                res
            };
            points.push(point!(x, y));
        }

        points
    }

    pub fn unpack(&self, points: &Vec<Point>) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        for p in points.iter() {
            let block = p.x.to_bytes()[1..].to_vec();
            res.extend(&block);
        }
        res
    }
}
