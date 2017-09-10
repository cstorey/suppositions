use std::fmt;
use hex_slice::AsHex;
use rand::random;

#[derive(Clone, Default)]
pub struct InfoPool {
    data: Vec<u8>,
    off: usize,
}

impl fmt::Debug for InfoPool {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("InfoPool")
            .field("data", &format_args!("{:x}", self.data.as_hex()))
            .field("off", &self.off)
            .finish()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PoolExhausted;

pub type Maybe<T> = Result<T, PoolExhausted>;


impl InfoPool {
    pub fn of_vec(data: Vec<u8>) -> Self {
        InfoPool { data: data, off: 0 }
    }
    pub fn random_of_size(size: usize) -> Self {
        Self::of_vec((0..size).map(|_| random()).collect::<Vec<u8>>())
    }

    pub fn next_byte(&mut self) -> Maybe<u8> {
        let res = self.data.get(self.off).cloned();
        self.off += 1;
        res.ok_or(PoolExhausted)
    }
    pub fn reset(&mut self) {
        self.off = 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_take_each_item_in_pool() {
        let mut p = InfoPool::of_vec(vec![0, 1, 2, 3]);
        assert_eq!(p.next_byte(), Ok(0));
        assert_eq!(p.next_byte(), Ok(1));
        assert_eq!(p.next_byte(), Ok(2));
        assert_eq!(p.next_byte(), Ok(3));
        assert_eq!(p.next_byte(), Err(PoolExhausted));
    }

    #[test]
    fn should_generate_random_data_of_size() {
        let size = 100;
        let mut p = InfoPool::random_of_size(size);
        for _ in 0..size {
            assert!(p.next_byte().is_ok());
        }
        assert!(p.next_byte().is_err());
    }

    #[test]
    fn should_allow_restarting_read() {
        let mut p = InfoPool::random_of_size(4);
        let mut v0 = Vec::new();
        while let Ok(val) = p.next_byte() {
            v0.push(val)
        }

        p.reset();
        let mut v1 = Vec::new();
        while let Ok(val) = p.next_byte() {
            v1.push(val)
        }

        assert_eq!(v0, v1)
    }


}
