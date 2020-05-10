use std::cmp;
use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};
use crate::{point};
use crate::base::{Point, CurveTrait};


pub struct Mapper<T: CurveTrait> {
    block_size: usize,
    curve: T
}


impl<T: CurveTrait + Copy> Mapper<T> {
    pub fn new(bits: usize, curve: &T) -> Self {
        Self {
            block_size: bits / 8 - 2,
            curve: *curve
        }
    }

    pub fn pack(&self, body: &Vec<u8>) -> Vec<Point> {
        let size = body.len();
        let mut points: Vec<Point> = Vec::new();

        for i in (0..size).step_by(self.block_size) {
            let end = cmp::min(i + self.block_size, size);

            let block = &body[i..end];
            let (x, y) = {
                let res;
                let mut x = Bigi::from_bytes(&block) << 8;
                loop {
                    match self.curve.find_y(&x) {
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
            let block = p.x.to_bytes()[1..(self.block_size + 1)].to_vec();
            res.extend(&block);
        }
        if let Some(idx) = res.iter().rposition(|e| *e != 0) {
            let end = idx + 1;
            res.truncate(end);
        }
        res
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use crate::schemas::load_secp256k1;

    #[test]
    fn test_mapper() {
        let body = "use bigi::{Bigi, bigi, BIGI_MAX_DIGITS};".as_bytes().to_vec();
        let schema = load_secp256k1();
        let mapper = Mapper::new(256, &schema.curve);

        let points = mapper.pack(&body);

        assert_eq!(points.len(), 2);

        let unpacked = mapper.unpack(&points);

        assert_eq!(unpacked, body);
    }

    #[bench]
    fn bench_pack_1024(b: &mut Bencher) {
        let body: Vec<u8> = (0..1024).map(|_| { rand::random::<u8>() }).collect();
        let schema = load_secp256k1();
        let mapper = Mapper::new(256, &schema.curve);
        b.iter(|| mapper.pack(&body));
    }

    #[bench]
    fn bench_unpack_1024(b: &mut Bencher) {
        let body: Vec<u8> = (0..1024).map(|_| { rand::random::<u8>() }).collect();
        let schema = load_secp256k1();
        let mapper = Mapper::new(256, &schema.curve);
        let points = mapper.pack(&body);
        b.iter(|| mapper.unpack(&points));
    }
}
