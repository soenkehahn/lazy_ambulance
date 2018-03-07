use ringbuf::RingBuf;

use jack::*;

use std::f32::consts::PI;
use std::rc::Rc;

pub trait Generator: Send {
    fn next_sample(&mut self, _client: &Client) -> f32;
}

// * actual sound

pub fn generator() -> impl Generator {
    Delay::new(sin(add(mult(sin(konst(1.0)), konst(50.0)), konst(350.0))))
}

pub struct Sin<G: Generator> {
    pub freq: G,
    pub phase: f32,
}

impl<G: Generator> Sin<G> {
    fn new(freq: G) -> Sin<G> {
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

pub struct Add<A: Generator, B: Generator>(pub A, pub B);

impl<A: Generator, B: Generator> Generator for Add<A, B> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        self.0.next_sample(client) + self.1.next_sample(client)
    }
}

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

pub struct Delay<G: Generator> {
    pub input: G,
    pub buffer: RingBuf<f32>,
}

impl<G: Generator> Delay<G> {
    fn new(input: G) -> Delay<G> {
        let buffer = RingBuf::new(30000, 0.0);
        Delay { input, buffer }
    }
}

impl<G: Generator> Generator for Delay<G> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        let sample = self.input.next_sample(client);
        let old = self.buffer[0];
        self.buffer.push(sample);
        sample * 0.7 + old * 0.3
    }
}

pub struct Foo<'a> {
    next_fn: Box<FnMut(&Client) -> f32 + Send + 'a>,
}

impl<'a> Foo<'a> {
    fn next(&mut self, client: &Client) -> f32 {
        (*self.next_fn)(client)
    }
}

impl<'a> Generator for Foo<'a> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        self.next(client)
    }
}

pub fn konst<'a>(f: f32) -> Rc<Foo<'a>> {
    Rc::new(Foo {
        next_fn: Box::new(move |_| f),
    })
}

pub fn sin<'a>(freq: Rc<Foo<'a>>) -> Foo<'a> {
    let mut phase = 0.0f32;
    Rc::new(Foo {
        next_fn: Box::new(move |client| {
            let result = phase.sin();
            phase += freq.next(client) * 2.0 * PI / (client.sample_rate() as f32);
            phase %= 2.0 * PI;
            result
        }),
    })
}

pub fn add<'a>(lhs: Rc<Foo<'a>>, rhs: Rc<Foo<'a>>) -> Rc<Foo<'a>> {
    Rc::new(Foo {
        next_fn: Box::new(move |client| lhs.next(client) + rhs.next(client)),
    })
}

pub fn mult<'a>(lhs: Rc<Foo<'a>>, rhs: Rc<Foo<'a>>) -> Rc<Foo<'a>> {
    Rc::new(Foo {
        next_fn: Box::new(move |client| lhs.next(client) * rhs.next(client)),
    })
}
