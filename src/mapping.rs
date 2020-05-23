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

        let step = {
            let mut step = size / self.block_size;
            if size % self.block_size > 0 {
                step += 1;
            }
            step
        };

        (0..step).map(|idx| {
            let block: Vec<u8> = body[idx..].iter().step_by(step).cloned().collect();

            let mut x = Bigi::from_bytes(&block) << 8;
            let y;

            loop {
                match self.curve.find_y(&x) {
                    Ok(roots) => { y = roots.0; break; },
                    Err(_e) => { x += &bigi![1] }
                }
            }

            point!(x, y)
        }).collect()
    }

    pub fn unpack(&self, points: &Vec<Point>) -> Vec<u8> {
        let step = points.len();
        let mut res: Vec<u8> = vec![0u8; step * self.block_size];

        for (idx, p) in points.iter().enumerate() {
            let block = p.x.to_bytes()[1..(self.block_size + 1)].to_vec();
            for i in 0..self.block_size {
                res[idx + i * step] = block[i];
            }
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
