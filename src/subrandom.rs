// Implementation of SubRandom sequences

use rand::Rng;

pub struct VanderCorput(u64);

impl VanderCorput {
    pub fn new() -> Self {
        VanderCorput(0)
    }
}
impl Rng for VanderCorput {
    fn next_u64(&mut self) -> u64 {
        let &mut VanderCorput(ref mut n) = self;
        let mut res = 0;
        for src in 0..64 {
            let val = (*n).wrapping_shr(src) & 1;
            let dst = 63 - src;
            res = res | val.wrapping_shl(dst);
            println!("n: {:16x}; i:{}={}; res: {:16x}", *n, src, val, res);
        }
        *n += 1;
        res
    }
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_from_zero() {
        let mut g = VanderCorput::new();
        assert_eq!(g.next_u64(), 0b0);

        assert_eq!(g.next_u64(), 0b1 << 63);

        assert_eq!(g.next_u64(), 0b01 << 62);
        assert_eq!(g.next_u64(), 0b11 << 62);

        assert_eq!(g.next_u64(), 0b001 << 61);
        assert_eq!(g.next_u64(), 0b101 << 61);
        assert_eq!(g.next_u64(), 0b011 << 61);
        assert_eq!(g.next_u64(), 0b111 << 61);
    }

}
