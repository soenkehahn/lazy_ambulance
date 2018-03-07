use std::cmp::max;
use std::ops::{Index, IndexMut};
#[derive(Debug)]
pub struct RingBuf<T> {
    cap: usize,
    top: usize,
    buf: Vec<T>,
}

impl<T: Clone> RingBuf<T> {
    pub fn new(cap: usize, initial: T) -> Self {
        RingBuf {
            cap: max(cap, 1),
            top: 0,
            buf: vec![initial; cap],
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn push(&mut self, t: T) {
        if self.len() == self.cap {
            self.buf[self.top] = t;
            self.top = (self.top + 1) % self.cap;
        } else {
            self.buf.push(t)
        }
    }
}

impl<T> Index<usize> for RingBuf<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        self.buf.index((idx + self.top) % self.cap)
    }
}

impl<T> IndexMut<usize> for RingBuf<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.buf.index_mut((idx + self.top) % self.cap)
    }
}
