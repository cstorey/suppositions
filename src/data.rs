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
pub enum DataError {
    PoolExhausted,
    SkipItem,
}

pub type Maybe<T> = Result<T, DataError>;

impl InfoPool {
    pub fn of_vec(data: Vec<u8>) -> Self {
        InfoPool { data: data }
    }
    pub fn random_of_size(size: usize) -> Self {
        Self::of_vec((0..size).map(|_| random()).collect::<Vec<u8>>())
    }

    pub fn buffer(&self) -> &[u8] {
        &*self.data
    }

    pub fn tap(&self) -> InfoTap {
        InfoTap {
            data: &*self.data,
            off: 0,
        }
    }
}

impl<'a> InfoTap<'a> {
    pub fn next_byte(&mut self) -> Maybe<u8> {
        let res = self.data.get(self.off).cloned();
        self.off += 1;
        res.ok_or(DataError::PoolExhausted)
    }
}


impl<'a> Iterator for InfoTap<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.next_byte().ok()
    }
}

fn minimize_via_removal<F: Fn(InfoTap) -> bool>(
    p: &InfoPool,
    candidate: &mut InfoPool,
    pred: &F,
) -> Option<InfoPool> {
    // First shrink tactic: item removal
    trace!("minimizing by removal: {:?}", p);
    let max_pow = 0usize.count_zeros();
    let pow = max_pow - p.data.len().leading_zeros();
    for granularity in 0..pow {
        let width = p.data.len() >> granularity;
        for chunk in 0..(1 << granularity) {
            let start = chunk * width;
            let end = start + width;
            candidate.data.clear();
            candidate.data.extend(&p.data[0..start]);
            candidate.data.extend(&p.data[end..]);

            let test = pred(candidate.tap());
            trace!(
                "removed {},{}: {:?}; test result {}",
                start,
                end,
                candidate,
                test
            );
            if test {
                if let Some(res) = minimize(&candidate, pred) {
                    trace!("Returning shrunk: {:?}", res);
                    return Some(res);
                } else {
                    trace!("Returning original: {:?}", candidate);
                    return Some(candidate.clone());
                }
            }
        }
    }
    None
}

fn minimize_via_scalar_shrink<F: Fn(InfoTap) -> bool>(
    p: &InfoPool,
    candidate: &mut InfoPool,
    pred: &F,
) -> Option<InfoPool> {
    // Second shrink tactic: make values smaller
    trace!("minimizing by scalar shrink: {:?}", p);
    for i in 0..p.data.len() {
        candidate.clone_from(&p);

        for bitoff in 0..8 {
            candidate.data[i] = p.data[i] - (p.data[i] >> bitoff);
            trace!(
                "shrunk item -(bitoff:{}) {} {}->{}: {:?}",
                bitoff,
                i,
                p.data[i],
                candidate.data[i],
                candidate
            );

            if candidate.buffer() == p.buffer() {
                trace!("No change");
                continue;
            }

            let test = pred(candidate.tap());
            trace!("test result {}", test);
            if test {
                if let Some(res) = minimize(&candidate, pred) {
                    trace!("Returning shrunk: {:?}", res);
                    return Some(res);
                } else {
                    trace!("Returning original: {:?}", candidate);
                    return Some(candidate.clone());
                }
            }
        }
    }

    None
}

pub fn minimize<F: Fn(InfoTap) -> bool>(p: &InfoPool, pred: &F) -> Option<InfoPool> {
    let mut candidate = p.clone();
    if let Some(res) = minimize_via_removal(p, &mut candidate, pred) {
        return Some(res);
    }

    if let Some(res) = minimize_via_scalar_shrink(p, &mut candidate, pred) {
        return Some(res);
    }

    trace!("Nothing smaller found than {:?}", p);
    None
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
        assert_eq!(t.next_byte(), Err(DataError::PoolExhausted));
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

    #[test]
    fn should_allow_borrowing_buffer() {
        let p = InfoPool::of_vec(vec![1]);
        assert_eq!(p.buffer(), &[1]);
    }

    #[test]
    fn tap_can_act_as_iterator() {
        let buf = vec![4, 3, 2, 1];
        let p = InfoPool::of_vec(buf.clone());
        let _: &Iterator<Item = u8> = &p.tap();

        assert_eq!(p.tap().collect::<Vec<_>>(), buf)
    }
    #[test]
    fn minimiser_should_minimise_to_empty() {
        let p = InfoPool::of_vec(vec![1]);
        let min = minimize(&p, &|_| true);

        assert_eq!(min.as_ref().map(|p| p.buffer()), Some([].as_ref()))
    }

    #[test]
    fn minimiser_should_minimise_to_minimum_given_size() {
        let p = InfoPool::of_vec(vec![0; 4]);
        let min = minimize(&p, &|t| t.count() > 1).expect("some smaller pool");

        assert_eq!(min.buffer(), &[0, 0])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|mut t| t.any(|v| v >= 3)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[3])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_to_zero() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|mut t| t.any(|_| true)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[0])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values_by_search() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|mut t| t.any(|v| v >= 13)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[13])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_accounting_for_overflow() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|mut t| t.any(|v| v >= 251)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[251])
    }
}
