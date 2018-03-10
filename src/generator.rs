use ringbuf::RingBuf;

use jack::*;

use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub trait Generator: Send + Clone {
    fn next_sample(&mut self, _client: &Client) -> f32;
}

// * actual sound

#[derive(Clone)]
pub struct Sin<G: Generator> {
    pub freq: G,
    pub phase: f32,
}

impl<G: Generator> Sin<G> {
    pub fn new(freq: G) -> Sin<G> {
        Sin {
            freq: freq,
            phase: 0.0,
        }
    }
}

impl<G: Generator> Generator for Sin<G> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        let result = self.phase.sin();
        self.phase += self.freq.next_sample(client) * 2.0 * PI / (client.sample_rate() as f32);
        self.phase %= 2.0 * PI;
        result
    }
}

#[derive(Clone)]
pub struct Add<A: Generator, B: Generator>(pub A, pub B);

impl<A: Generator, B: Generator> Generator for Add<A, B> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        self.0.next_sample(client) + self.1.next_sample(client)
    }
}

#[derive(Clone)]
pub struct Mult<A: Generator, B: Generator>(pub A, pub B);

impl<A: Generator, B: Generator> Generator for Mult<A, B> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        self.0.next_sample(client) * self.1.next_sample(client)
    }
}

impl Generator for f32 {
    fn next_sample(&mut self, _client: &Client) -> f32 {
        *self
    }
}

#[derive(Clone)]
pub struct Variable(Arc<AtomicU32>);

impl Variable {
    pub fn new(pitch: Arc<AtomicU32>) -> Variable {
        Variable(pitch)
    }
}

impl Generator for Variable {
    fn next_sample(&mut self, _client: &Client) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
}

// #[derive(Clone, Copy)]
// pub struct Delay<G: Generator> {
//     pub input: G,
//     pub buffer: RingBuf<f32>,
// }

// impl<G: Generator> Delay<G> {
//     pub fn new(input: G) -> Delay<G> {
//         Self::new_bufsize(input, 30000)
//     }

//     pub fn new_bufsize(input: G, bufsize: usize) -> Delay<G> {
//         let buffer = RingBuf::new(bufsize, 0.0);
//         Delay { input, buffer }
//     }
// }

// impl<G: Generator> Generator for Delay<G> {
//     fn next_sample(&mut self, client: &Client) -> f32 {
//         let sample = self.input.next_sample(client);
//         let old = self.buffer[0];
//         self.buffer.push(sample);
//         sample * 0.7 + old * 0.3
//     }
// }
