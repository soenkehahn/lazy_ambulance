#![feature(allocator_api)]
#![feature(type_ascription)]
#![feature(optin_builtin_traits)]
#![feature(conservative_impl_trait)]

extern crate jack;
extern crate leaker;
use jack::*;
use std::f32::consts::PI;
use leaker::*;

struct NH;

impl NotificationHandler for NH {}

struct PH<G: Generator> {
    ports: Vec<Port<AudioOut>>,
    generator: G,
}

trait Generator: Send {
    fn next_sample(&mut self, _client: &Client) -> f32;
}

impl<G: Generator> ProcessHandler for PH<G> {
    fn process(&mut self, client: &Client, scope: &ProcessScope) -> Control {
        for sample_index in 0..scope.n_frames() {
            let sample = self.generator.next_sample(client);
            for port in self.ports.iter_mut() {
                let slice = port.as_mut_slice(scope);
                slice[sample_index as usize] = sample;
            }
        }
        Control::Continue
    }
}

fn main_() -> Result<(), jack::Error> {
    let (client, _status) = Client::new("rusty_client", jack::ClientOptions::NO_START_SERVER)?;
    let mut outputs = vec![];
    outputs.push(client.register_port("output1", AudioOut)?);
    outputs.push(client.register_port("output2", AudioOut)?);

    let _async_client = client.activate_async(
        NH,
        PH {
            ports: outputs,
            generator: generator(),
        },
    )?;

    std::thread::sleep(std::time::Duration::new(50, 0));

    Ok(())
}

fn main() {
    main_().unwrap();
}

// * actual sound

fn generator() -> impl Generator {
    Delay::new(Sin::new(Add(Mult(Sin::new(1.0), 50.0), 350.0)))
}

struct Sin<G: Generator> {
    freq: G,
    phase: f32,
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

struct Add<A: Generator, B: Generator>(A, B);

impl<A: Generator, B: Generator> Generator for Add<A, B> {
    fn next_sample(&mut self, client: &Client) -> f32 {
        self.0.next_sample(client) + self.1.next_sample(client)
    }
}

struct Mult<A: Generator, B: Generator>(A, B);

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

struct Delay<G: Generator> {
    input: G,
    buffer: RingBuf<f32>,
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
