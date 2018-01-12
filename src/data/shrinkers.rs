use std::cmp::min;
use data::source::*;
use std::collections::HashSet;

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
struct DeltaDebugSegmentIterator {
    size: usize,
    log2sz: usize,
    level: usize,
    chunk: usize,
}

#[derive(Debug)]
struct RemovalShrinker<I> {
    seed: InfoPool,
    segments: I,
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
pub fn minimize<F: Fn(&mut InfoRecorder<InfoReplay>) -> bool>(
    orig: &InfoPool,
    pred: &F,
) -> InfoPool {
    let mut best = orig.clone();
    // this might be better as something that we can apply a window to,
    // or bloom filter.
    let mut seen = HashSet::new();

    loop {
        debug!("Shrinking pool");
        debug!("Seen {:?} pools", seen.len());
        trace!("Pool: {:?}", best);

        let interval_removals = RemovalShrinker::remove_recorded_intervals(best.clone());
        let delta_removals = RemovalShrinker::delta_debug_of_pool(best.clone());
        let scalars = ScalarShrinker::new(best.clone());
        let shrunk_pools = interval_removals.chain(delta_removals).chain(scalars);

        {
            let mut matching_shrinks = shrunk_pools.filter_map(|c| {
                if seen.contains(&c) {
                    debug!("Skipping seen item");
                    return None;
                }
                seen.insert(c.clone());

                let mut recorder = InfoRecorder::new(c.replay());
                let test = pred(&mut recorder);
                trace!("test result: {:?} <= {:?}", test, c);
                // Extract the execution trace from the pool at this point.
                if test {
                    Some(recorder.into_pool())
                } else {
                    None
                }
            });

            if let Some(candidate) = matching_shrinks.next() {
                debug!("Re-Shrinking");
                best = candidate;
            } else {
                debug!("Nothing smaller found");
                trace!("... than {:?}", best);
                break;
            }
        }

        trace!("Note best: {:?}", best);
    }

    best
}

fn ulog2(val: usize) -> usize {
    let max_pow = 0usize.count_zeros() as usize;
    max_pow - val.leading_zeros() as usize
}

impl DeltaDebugSegmentIterator {
    fn new(size: usize) -> Self {
        let log2sz = ulog2(size.saturating_sub(1));
        let level = 0;
        let chunk = 0;
        DeltaDebugSegmentIterator {
            size,
            log2sz,
            level,
            chunk,
        }
    }
}

impl Iterator for DeltaDebugSegmentIterator {
    type Item = Span;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            trace!("DeltaDebugSegmentIterator#next: {:?}", self);
            if self.level > self.log2sz {
                return None;
            }

            let granularity = self.log2sz - self.level;
            let width = 1 << granularity;
            let chunk = self.chunk;

            let start = chunk * width;
            let end = start + width;

            if start >= self.size {
                trace!("Out of slice ({},{}) >= {}", start, end, self.size);
                self.chunk = 0;
                self.level += 1;
                continue;
            } else {
                self.chunk += 1;
            }
            let start = min(start, self.size);
            let end = min(end, self.size);

            debug!("DeltaDebugSegmentIterator::next() -> ({},{})", start, end);

            return Some(Span::of_pair((start, end)));
        }
    }
}

impl<I> RemovalShrinker<I> {
    fn new(seed: InfoPool, segments: I) -> Self {
        RemovalShrinker { seed, segments }
    }
}

impl RemovalShrinker<InfoPoolIntervalsIter> {
    fn remove_recorded_intervals(seed: InfoPool) -> Self {
        let segments = seed.spans_iter();
        RemovalShrinker::new(seed, segments)
    }
}

impl RemovalShrinker<DeltaDebugSegmentIterator> {
    fn delta_debug_of_pool(seed: InfoPool) -> Self {
        let len = seed.data.len();
        RemovalShrinker::new(seed, DeltaDebugSegmentIterator::new(len))
    }
}

impl<I: Iterator<Item = Span>> Iterator for RemovalShrinker<I> {
    type Item = InfoPool;
    fn next(&mut self) -> Option<Self::Item> {
        self.segments.next().map(|span| {
            // let start = min(start, self.seed.data.len());
            // let end = min(end, self.seed.data.len());

            let mut candidate = InfoPool::new();
            candidate.data.clear();
            candidate.data.extend(&self.seed.data[span.before()]);
            candidate.data.extend(&self.seed.data[span.after()]);
            debug!("removed {:?}", span);
            trace!("candidate {:?}", candidate);

            candidate
        })
    }
}
#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;
    use std::collections::BTreeMap;

    // The end of the buffer is semantically equivalent to zero for generators,
    // so we can ignore those.
    fn without_trailing_zeroes(mut buf: &[u8]) -> &[u8] {
        while buf.last() == Some(&0) {
            let l = buf.len();
            buf = &buf[..l - 1]
        }

        buf
    }

    #[test]
    fn minimiser_should_minimise_to_empty() {
        let p = InfoPool::of_vec(vec![1]);
        let min = minimize(&p, &|_| true);

        assert_eq!(min.buffer(), &[0u8; 0])
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
        });

        assert_eq!(without_trailing_zeroes(min.buffer()), &[1, 1])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 3));

        assert_eq!(without_trailing_zeroes(min.buffer()), &[3])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_to_empty() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|_| true));

        assert_eq!(without_trailing_zeroes(min.buffer()), &[] as &[u8])
    }

    #[test]
    fn minimiser_should_minimise_scalar_values_by_search() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 13));

        assert_eq!(without_trailing_zeroes(min.buffer()), &[13])
    }
    #[test]
    fn minimiser_should_minimise_scalar_values_accounting_for_overflow() {
        let p = InfoPool::of_vec(vec![255; 3]);
        let min = minimize(&p, &|t| take_n(t, 16).into_iter().any(|v| v >= 251));

        assert_eq!(without_trailing_zeroes(min.buffer()), &[251])
    }

    #[test]
    fn shrink_by_delta_debug_removal_should_produce_somewhat_unique_outputs() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec((0..256usize).map(|v| v as u8).collect::<Vec<_>>());
        let mut counts = BTreeMap::new();
        for val in RemovalShrinker::delta_debug_of_pool(p) {
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

    #[test]
    fn delta_debug_segments_should_generate_segments_on_power_of_two_boundary() {
        use std::collections::BTreeSet;
        let iter = DeltaDebugSegmentIterator::new(7);
        let items = iter.collect::<BTreeSet<_>>();
        assert_eq!(
            items,
            vec![
                (0, 7),
                (0, 4),
                (4, 7),
                (0, 2),
                (2, 4),
                (4, 6),
                (6, 7),
                (0, 1),
                (2, 3),
                (4, 5),
                (6, 7),
                (1, 2),
                (3, 4),
                (5, 6),
            ].into_iter().map(Span::of_pair)
                .collect()
        );
    }

    #[test]
    fn delta_debug_segments_should_generate_segments_of_non_increasing_size() {
        let lengths = DeltaDebugSegmentIterator::new(7)
            .map(|span| { let (start, end) = span.as_pair(); end - start })
            .collect::<Vec<_>>();

        assert!(
            lengths
                .iter()
                .zip(lengths.iter().skip(1))
                .all(|(a, b)| a >= b),
            "Generated lengths {:?} are monotonically decreasing",
            lengths
        );
    }

    #[test]
    fn shrink_by_removal_should_remove_stated_slices() {
        env_logger::init().unwrap_or(());
        let p = InfoPool::of_vec(vec![0, 1, 2, 3, 4]);
        let vals = RemovalShrinker::new(p, ::std::iter::once(Span::of_pair((2, 3)))).collect::<Vec<_>>();

        assert_eq!(vals, vec![InfoPool::of_vec(vec![0, 1, 3, 4])]);
    }
}
