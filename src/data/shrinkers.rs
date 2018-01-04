use std::cmp::min;
use data::source::*;

/// Iterates over a series of shrunk pools. If we imagine that our buffer has
/// a sz of (1 << (log2sz-1)) < sz ≤ (1 << log2sz), then where:
/// f(x) = 1<<(log2sz-x) we want to cut out
/// chunks of:
/// ```
/// 0..f(0),
/// 0..f(1), f(1)..2f(1),
/// 0..f(2), f(2)..2f(2), 2f(2)..3f(2), 3f(2)..4f(2),
/// ```
///
/// In other words, we remove the whole lot, then first half, second half,
/// first quarter, second quarter, etc.
#[derive(Debug)]
struct RemovalShrinker {
    seed: InfoPool,
    log2sz: usize,
    level: usize,
    chunk: usize,
}

impl RemovalShrinker {
    fn new(seed: InfoPool) -> Self {
        let max_idx = seed.data.len().saturating_sub(1);
        let max_pow = 0usize.count_zeros();
        let pow = max_pow - max_idx.leading_zeros();
        RemovalShrinker {
            seed,
            log2sz: pow as usize,
            // Ranges from 0..self.log2sz
            level: 0,
            // Ranges from 0..(1<<self.level)
            chunk: 0,
        }
    }
}

impl Iterator for RemovalShrinker {
    type Item = InfoPool;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            trace!("RemovalShrinker#next: {:?}", self);
            if self.level > self.log2sz {
                return None;
            }

            let granularity = self.log2sz - self.level;
            let width = 1 << granularity;
            let chunk = self.chunk;

            let start = chunk * width;
            let end = start + width;

            if start >= self.seed.data.len() {
                trace!(
                    "Out of slice ({},{}) >= {}",
                    start,
                    end,
                    self.seed.data.len()
                );
                self.chunk = 0;
                self.level += 1;
                continue;
            } else {
                self.chunk += 1;
            }
            let start = min(start, self.seed.data.len());
            let end = min(end, self.seed.data.len());

            let mut candidate = InfoPool::new();
            candidate.data.clear();
            candidate.data.extend(&self.seed.data[0..start]);
            candidate.data.extend(&self.seed.data[end..]);
            debug!("removed {},{}", start, end);
            trace!("candidate {:?}", candidate);

            return Some(candidate);
        }
    }
}

#[derive(Debug)]
struct ScalarShrinker {
    seed: InfoPool,
    pos: usize,
    bitoff: u32,
}

impl ScalarShrinker {
    fn new(pool: InfoPool) -> Self {
        ScalarShrinker {
            seed: pool,
            pos: 0,
            bitoff: 0,
        }
    }
}
impl Iterator for ScalarShrinker {
    type Item = InfoPool;
    fn next(&mut self) -> Option<Self::Item> {
        let mut candidate = self.seed.clone();
        while self.pos < self.seed.data.len() {
            trace!("ScalarShrinker#next: {:?}", self);
            let pos = self.pos;
            let bitoff = self.bitoff;

            let orig_val = candidate.data[pos];
            let new_val = orig_val - (orig_val.overflowing_shr(bitoff).0);

            let log2val = (0u8.leading_zeros()) - orig_val.leading_zeros();
            if self.bitoff >= log2val {
                self.pos += 1;
                self.bitoff = 0;
                continue;
            } else {
                self.bitoff += 1;
            }

            if orig_val != new_val {
                candidate.data[pos] = new_val;
                debug!(
                    "shrunk item -(bitoff:{}) {} {}->{}",
                    bitoff, pos, orig_val, new_val
                );
                trace!("candidate {:?}", candidate);

                return Some(candidate);
            }
        }
        return None;
    }
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
pub fn minimize<F: Fn(InfoRecorder<InfoReplay>) -> bool>(p: &InfoPool, pred: &F) -> Option<InfoPool> {
    let shrunk_pools = RemovalShrinker::new(p.clone()).chain(ScalarShrinker::new(p.clone()));

    debug!("Shrinking pool");
    let mut matching_shrinks = shrunk_pools.filter(|c| {
        let pool = InfoRecorder::new(c.replay());
        let test = pred(pool);
        trace!("test result: {:?} <= {:?}", test, c);
        test
    });

    if let Some(candidate) = matching_shrinks.next() {
        debug!("Re-Shrinking");
        trace!("candidate {:?}", candidate);
        let result = minimize(&candidate, pred).unwrap_or(candidate);
        debug!("Re-Shrinking done");
        Some(result)
    } else {
        debug!("Nothing smaller found");
        trace!("... than {:?}", p);
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn minimiser_should_minimise_to_empty() {
        let p = InfoPool::of_vec(vec![1]);
        let min = minimize(&p, &|_| true);

        assert_eq!(min.as_ref().map(|p| p.buffer()), Some([].as_ref()))
    }

    fn take_n<I: InfoSource>(mut src: I, n: usize) -> Vec<u8> {
        let mut res = Vec::new();
        for _ in 0..n {
            res.push(src.draw_u8())
        }
        res
    }

    #[test]
    fn minimiser_should_minimise_to_minimum_given_size() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec(vec![1; 4]);
        let min = minimize(&p, &|t| {
            take_n(t, 16).into_iter().filter(|&v| v > 0).count() > 1
        }).expect("some smaller pool");

        assert_eq!(min.buffer(), &[1, 1])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 3))
            .expect("some smaller pool");

        assert_eq!(min.buffer(), &[3])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_to_empty() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min =
            minimize(&p, &|t| take_n(t, 16).into_iter().any(|_| true)).expect("some smaller pool");

        assert_eq!(min.buffer(), &[] as &[u8])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values_by_search() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 13))
            .expect("some smaller pool");

        assert_eq!(min.buffer(), &[13])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_accounting_for_overflow() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 251))
            .expect("some smaller pool");

        assert_eq!(min.buffer(), &[251])
    }

    #[test]
    fn shrink_by_removal_should_produce_somewhat_unique_outputs() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec((0..256usize).map(|v| v as u8).collect::<Vec<_>>());
        let mut counts = BTreeMap::new();
        for val in RemovalShrinker::new(p) {
            debug!("{:?}", val);
            *counts.entry(val).or_insert(0) += 1;
        }

        assert!(
            counts.values().all(|&val| val == 1),
            "Expect all items to be unique; non-unique entries {:?}",
            counts
                .iter()
                .filter(|&(_, &v)| v != 1)
                .collect::<BTreeMap<_, _>>()
        )
    }

    #[test]
    fn shrink_by_scalar_should_produce_somewhat_unique_outputs() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec((0..256usize).map(|v| v as u8).collect::<Vec<_>>());
        let mut counts = BTreeMap::new();
        for val in ScalarShrinker::new(p) {
            let ent = counts.entry(val.clone()).or_insert(0);
            *ent += 1;
            if *ent > 1 {
                debug!("Dup! {}: {:?}", *ent, val);
            }
        }

        assert!(
            counts.values().all(|&val| val == 1),
            "Expect all items to be unique; non-unique entries {:?}",
            counts
                .iter()
                .filter(|&(_, &v)| v != 1)
                .collect::<BTreeMap<_, _>>()
        )
    }

}
