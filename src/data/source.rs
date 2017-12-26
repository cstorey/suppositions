use std::fmt;
use hex_slice::AsHex;
use rand::{random, Rng, XorShiftRng};
use std::rc::Rc;
use std::cell::RefCell;

/// Something that can extract information from an `InfoSource`.
pub trait InfoSink {
    /// The output data.
    type Out;
    /// Called by [InfoSource::draw](trait.InfoSource.html#tymethod.draw)
    fn sink<I: InfoSource>(&mut self, i: &mut I) -> Self::Out;
}

/// Something that an act as a source of test data.
pub trait InfoSource {
    /// Take a single byte from the source.
    fn draw_u8(&mut self) -> u8;

    /// Call F with access to the data source.
    fn draw<S: InfoSink>(&mut self, sink: S) -> S::Out
    where
        Self: Sized;
}

/// Generates data from an underlying Rng instance.
#[derive(Debug)]
pub struct RngSource<R> {
    rng: R,
}

#[derive(Debug)]
pub(crate) enum DataChunk {
    Leaf(Vec<u8>),
    Branch(Vec<DataChunk>),
}

/// An adapter that can record the data drawn from an underlying source.
#[derive(Debug)]
pub struct InfoRecorder<I> {
    inner: Rc<RefCell<I>>,
    pub(crate) data: Vec<DataChunk>,
}

impl<'a, I: InfoSource + ?Sized> InfoSource for &'a mut I {
    fn draw_u8(&mut self) -> u8 {
        (**self).draw_u8()
    }
    fn draw<S: InfoSink>(&mut self, mut sink: S) -> S::Out
    where
        Self: Sized,
    {
        sink.sink(self)
    }
}

impl<I> InfoRecorder<I> {
    /// Creates a recording InfoSource.
    pub fn new(inner: I) -> Self {
        InfoRecorder {
            inner: Rc::new(RefCell::new(inner)),
            data: Vec::new(),
        }
    }

    /// Extracts the data recorded.
    pub fn into_pool(self) -> InfoPool {
        let mut data = Vec::new();
        self.flatten_to(&mut data);
        InfoPool::of_vec(data)
    }

    // This will go away once we integrate the tree structure into the
    // shrinky bits.
    fn flatten_to(&self, dst: &mut Vec<u8>) {
        for chunk in self.data.iter() {
            chunk.flatten_to(dst)
        }
    }

    fn into_chunk(self) -> DataChunk {
        DataChunk::Branch(self.data)
    }
}

impl DataChunk {
    fn flatten_to(&self, dst: &mut Vec<u8>) {
        match self {
            &DataChunk::Leaf(ref v) => dst.extend(v),
            &DataChunk::Branch(ref brs) => {
                for br in brs {
                    br.flatten_to(dst)
                }
            }
        }
    }
}

impl<I: InfoSource> InfoSource for InfoRecorder<I> {
    fn draw_u8(&mut self) -> u8 {
        let byte = self.inner.borrow_mut().draw_u8();
        let last_elt = self.data.pop().unwrap_or_else(|| DataChunk::Leaf(vec![]));

        let (prevp, last_elt) = match last_elt {
            DataChunk::Leaf(mut v) => {
                v.push(byte);
                (None, DataChunk::Leaf(v))
            }
            br @ DataChunk::Branch(_) => (Some(br), DataChunk::Leaf(vec![byte])),
        };

        if let Some(prev) = prevp {
            self.data.push(prev);
        };
        self.data.push(last_elt);
        byte
    }

    fn draw<S: InfoSink>(&mut self, mut sink: S) -> S::Out
    where
        Self: Sized,
    {
        let mut child = InfoRecorder {
            inner: self.inner.clone(),
            data: Vec::new(),
        };
        let res = sink.sink(&mut child);
        self.data.push(child.into_chunk());
        res
    }
}

impl RngSource<XorShiftRng> {
    /// Creates a RngSource with a randomly seeded XorShift generator.
    pub fn new() -> Self {
        let rng = random::<XorShiftRng>();
        RngSource { rng }
    }
}

impl<R: Rng> InfoSource for RngSource<R> {
    fn draw_u8(&mut self) -> u8 {
        self.rng.gen::<u8>()
    }
    fn draw<S: InfoSink>(&mut self, mut sink: S) -> S::Out
    where
        Self: Sized,
    {
        sink.sink(self)
    }
}

/// A pool of data that we can draw upon to generate other types of data.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InfoPool {
    pub(in data) data: Vec<u8>,
}

/// A handle to an info Pool that we can draw replayed bytes from, and zero after.
#[derive(Clone)]
pub struct InfoReplay<'a> {
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

    /// Create an `InfoPool` with a `size` length vector of random bytes
    /// using the generator `rng`. (Mostly used for testing).
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }


    /// Allows access to the underlying buffer.
    pub fn buffer(&self) -> &[u8] {
        &*self.data
    }

    /// Creates a tap that allows drawing new information from this pool.
    pub fn replay(&self) -> InfoReplay {
        InfoReplay {
            data: &*self.data,
            off: 0,
        }
    }
}

impl<'a> InfoReplay<'a> {
    /// Consumes the next byte from this tap. Returns `Ok(x)` if successful,
    /// or `Err(DataError::PoolExhausted)` if we have reached the end.
    pub fn next_byte(&mut self) -> u8 {
        let res = self.data.get(self.off).cloned();
        self.off += 1;
        res.unwrap_or(0)
    }
}


impl<'a> InfoSource for InfoReplay<'a> {
    fn draw_u8(&mut self) -> u8 {
        self.next_byte()
    }

    fn draw<S: InfoSink>(&mut self, mut sink: S) -> S::Out
    where
        Self: Sized,
    {
        sink.sink(self)
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;
    impl<R: Rng> RngSource<R> {
        pub(crate) fn of(rng: R) -> Self {
            RngSource { rng }
        }
    }

    #[test]
    fn should_take_each_item_in_pool() {
        let p = InfoPool::of_vec(vec![0, 1, 2, 3]);
        let mut t = p.replay();
        assert_eq!(t.next_byte(), 0);
        assert_eq!(t.next_byte(), 1);
        assert_eq!(t.next_byte(), 2);
        assert_eq!(t.next_byte(), 3);
        assert_eq!(t.next_byte(), 0);
    }

    #[test]
    fn should_generate_random_data() {
        let trials = 1024usize;
        let mut vals = 0;
        let mut p = RngSource::of(XorShiftRng::new_unseeded());
        let error = 8;
        for _ in 0..trials {
            vals += p.draw_u8() as usize;
        }
        let mean = vals / trials;
        let expected = 128;
        assert!(
            (expected - error) < mean && (expected + error) > mean,
            "Expected {} trials to be ({}+/-{}); got {}",
            trials,
            expected,
            error,
            mean
        )
    }

    #[test]
    fn should_allow_restarting_read() {
        let mut p = InfoRecorder::new(RngSource::new());
        let mut v0 = Vec::new();
        {
            for _ in 0..4 {
                v0.push(p.draw_u8())
            }
        }

        let p = p.into_pool();
        let mut t = p.replay();
        let mut v1 = Vec::new();
        for _ in 0..4 {
            v1.push(t.draw_u8())
        }

        assert_eq!(v0, v1)
    }

    #[test]
    fn should_allow_restarting_child_reads() {
        struct FnSink<F>(F);
        impl<F: FnMut(&mut InfoSource) -> R, R> InfoSink for FnSink<F> {
            type Out = R;
            fn sink<I: InfoSource>(&mut self, k: &mut I) -> R {
                (self.0)(k as &mut InfoSource)
            }
        }

        let mut p = InfoRecorder::new(RngSource::new());
        let mut v0 = Vec::new();
        p.draw(FnSink(|src: &mut InfoSource| for _ in 0..4 {
            let x: u8 = src.draw_u8();
            v0.push(x);
        }));

        let p = p.into_pool();
        let mut t = p.replay();
        let mut v1 = Vec::new();
        for _ in 0..4 {
            v1.push(t.draw_u8())
        }

        assert_eq!(v0, v1)
    }

    #[test]
    fn should_allow_restarting_mixed_child_reads() {
        struct FnSink<F>(F);
        impl<F: FnMut(&mut InfoSource) -> R, R> InfoSink for FnSink<F> {
            type Out = R;
            fn sink<I: InfoSource>(&mut self, k: &mut I) -> R {
                (self.0)(k as &mut InfoSource)
            }
        }

        let mut p = InfoRecorder::new(RngSource::new());
        let mut v0 = Vec::new();

        for _ in 0..2 {
            let x: u8 = p.draw_u8();
            v0.push(x);
        }

        p.draw(FnSink(|src: &mut InfoSource| for _ in 0..4 {
            let x: u8 = src.draw_u8();
            v0.push(x);
        }));

        for _ in 0..2 {
            let x: u8 = p.draw_u8();
            v0.push(x);
        }

        let p = p.into_pool();
        let mut t = p.replay();
        let mut v1 = Vec::new();
        for _ in 0..8 {
            v1.push(t.draw_u8())
        }

        assert_eq!(v0, v1)
    }


    #[test]
    fn should_allow_borrowing_buffer() {
        let p = InfoPool::of_vec(vec![1]);
        assert_eq!(p.buffer(), &[1]);
    }

    #[test]
    fn tap_can_act_as_source() {
        let buf = vec![4, 3, 2, 1];
        let p = InfoPool::of_vec(buf.clone());
        let _: &InfoSource = &p.replay();
        let mut res = Vec::new();
        let mut it = p.replay();
        for _ in 0..4 {
            res.push(it.draw_u8())
        }
        assert_eq!(res, buf)
    }

    #[test]
    fn replay_can_act_as_source() {
        let buf = vec![4, 3, 2, 1];
        let p = InfoPool::of_vec(buf.clone());

        let mut res = Vec::new();
        let mut it = p.replay();
        for _ in 0..4 {
            res.push(it.draw_u8())
        }
        assert_eq!(res, buf)
    }

}
