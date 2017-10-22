//! This module contains the underlying data generation and shrinking
//! mechanism. The main type is the `InfoPool`, which represents a pool of
//! random bytes that can be observed via the `InfoTap` object (obtained via
//! `InfoPool#tap`).
//!
//! Also manages the shrinking process (see [`minimize`](fn.minimize.html)).

use std::fmt;
use hex_slice::AsHex;
use rand::{random, Rng, Rand};

/// A pool of data that we can draw upon to generate other types of data.
#[derive(Clone, Default, PartialEq)]
pub struct InfoPool {
    data: Vec<u8>,
}

/// A handle to an info Pool that we can draw bytes from.
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

/// The reasons why drawing data from a pool can fail.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DataError {
    /// Not enough data to generate a value
    PoolExhausted,
    /// One of our combinators said that we should not test this value.
    SkipItem,
}


impl InfoPool {
    /// Create an `InfoPool` with a given vector of bytes. (Mostly used for
    /// testing).
    pub fn of_vec(data: Vec<u8>) -> Self {
        InfoPool { data: data }
    }
    /// Create an `InfoPool` with a `size` length vector of random bytes.
    /// (Mostly used for testing).
    pub fn random_of_size(size: usize) -> Self {
        Self::of_vec((0..size).map(|_| random()).collect::<Vec<u8>>())
    }

    /// Create an `InfoPool` with a `size` length vector of random bytes
    /// using the generator `rng`. (Mostly used for testing).
    pub fn from_random_of_size<R: Rng>(rng: &mut R, size: usize) -> Self {
        Self::of_vec(
            (0..size)
                .map(|_| (u64::rand(rng) >> 56) as u8)
                .collect::<Vec<u8>>(),
        )
    }

    /// Allows access to the underlying buffer.
    pub fn buffer(&self) -> &[u8] {
        &*self.data
    }

    /// Creates a tap that allows drawing information from this pool.
    pub fn tap(&self) -> InfoTap {
        InfoTap {
            data: &*self.data,
            off: 0,
        }
    }
}

impl<'a> InfoTap<'a> {
    /// Consumes the next byte from this tap. Returns `Ok(x)` if successful,
    /// or `Err(DataError::PoolExhausted)` if we have reached the end.
    pub fn next_byte(&mut self) -> u8 {
        let res = self.data.get(self.off).cloned();
        self.off += 1;
        res.unwrap_or(0)
    }
}


impl<'a> Iterator for InfoTap<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        Some(self.next_byte())
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

/// Try to find the smallest pool `p` such that the predicate `pred` returns
/// true. Given that our [generators](../generators/index.html) tend to
/// generate smaller outputs from smaller inputs, by minimizing the source
/// pool we can find the smallest value that provokes a failure.
///
/// If we do not find any smaller pool that satisifies `pred`; we return
/// `None`.
///
/// Currently, we have two heuristics for shrinking: removing slices, and
/// reducing individual values.
///
/// Removing slices tries to remove as much of the pool as it can whilst still
/// having the predicate hold. At present, we do this by removing each half
/// and testing, then each quarter, eighths, and so on. Eventually, we plan to
/// track the regions of data that each generator draws from, and use that to
/// optimise the shrinking process.
///
/// Reducing individual values basically goes through each position in the
/// pool, and then tries reducing it to zero, then half, thn three quarters,
/// seven eighths, and so on.
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
    extern crate env_logger;
    use super::*;

    #[test]
    fn should_take_each_item_in_pool() {
        let p = InfoPool::of_vec(vec![0, 1, 2, 3]);
        let mut t = p.tap();
        assert_eq!(t.next_byte(), 0);
        assert_eq!(t.next_byte(), 1);
        assert_eq!(t.next_byte(), 2);
        assert_eq!(t.next_byte(), 3);
        assert_eq!(t.next_byte(), 0);
    }

    #[test]
    fn should_generate_random_data_of_size() {
        let size = 100;
        let p = InfoPool::random_of_size(size);
        let mut t = p.tap();
        for _ in 0..size {
            let _ = t.next_byte();
        }
        assert_eq!(t.next_byte(), 0);
    }

    #[test]
    fn should_allow_restarting_read() {
        let p = InfoPool::random_of_size(4);
        let mut t = p.tap();
        let mut v0 = Vec::new();
        for _ in 0..4 {
            v0.push(t.next_byte())
        }

        let mut t = p.tap();
        let mut v1 = Vec::new();
        for _ in 0..4 {
            v1.push(t.next_byte())
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

        assert_eq!(p.tap().take(4).collect::<Vec<_>>(), buf)
    }
    #[test]
    fn minimiser_should_minimise_to_empty() {
        let p = InfoPool::of_vec(vec![1]);
        let min = minimize(&p, &|_| true);

        assert_eq!(min.as_ref().map(|p| p.buffer()), Some([].as_ref()))
    }

    #[test]
    fn minimiser_should_minimise_to_minimum_given_size() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec(vec![1; 4]);
        let min = minimize(&p, &|t| t.take(16).filter(|&v| v > 0).count() > 1).expect("some smaller pool");

        assert_eq!(min.buffer(), &[1, 1])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| t.take(16).any(|v| v >= 3)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[3])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_to_empty() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| t.take(16).any(|_| true)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[] as &[u8])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values_by_search() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| t.take(16).any(|v| v >= 13)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[13])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_accounting_for_overflow() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| t.take(16).any(|v| v >= 251)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[251])
    }
}
