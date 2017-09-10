use std::fmt;
use hex_slice::AsHex;
use rand::random;

#[derive(Clone, Default)]
pub struct InfoPool {
    data: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct InfoTap<'a> {
    data: &'a [u8],
    off: usize,
}

impl fmt::Debug for InfoPool {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("InfoPool")
            .field("data", &format_args!("{:x}", self.data.as_hex()))
            .finish()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PoolExhausted;

pub type Maybe<T> = Result<T, PoolExhausted>;


impl InfoPool {
    pub fn of_vec(data: Vec<u8>) -> Self {
        InfoPool { data: data }
    }
    pub fn random_of_size(size: usize) -> Self {
        Self::of_vec((0..size).map(|_| random()).collect::<Vec<u8>>())
    }

    pub fn tap(&self) -> InfoTap {
        InfoTap { data: &*self.data, off: 0 }
    } 
}

impl<'a> InfoTap<'a> {
    pub fn next_byte(&mut self) -> Maybe<u8> {
        let res = self.data.get(self.off).cloned();
        self.off += 1;
        res.ok_or(PoolExhausted)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_take_each_item_in_pool() {
        let p = InfoPool::of_vec(vec![0, 1, 2, 3]);
        let mut t = p.tap();
        assert_eq!(t.next_byte(), Ok(0));
        assert_eq!(t.next_byte(), Ok(1));
        assert_eq!(t.next_byte(), Ok(2));
        assert_eq!(t.next_byte(), Ok(3));
        assert_eq!(t.next_byte(), Err(PoolExhausted));
    }

    #[test]
    fn should_generate_random_data_of_size() {
        let size = 100;
        let p = InfoPool::random_of_size(size);
        let mut t = p.tap();
        for _ in 0..size {
            assert!(t.next_byte().is_ok());
        }
        assert!(t.next_byte().is_err());
    }

    #[test]
    fn should_allow_restarting_read() {
        let p = InfoPool::random_of_size(4);
        let mut t = p.tap();
        let mut v0 = Vec::new();
        while let Ok(val) = t.next_byte() {
            v0.push(val)
        }

        let mut t = p.tap();
        let mut v1 = Vec::new();
        while let Ok(val) = t.next_byte() {
            v1.push(val)
        }

        assert_eq!(v0, v1)
    }


}
