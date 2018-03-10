#![feature(integer_atomics)]
#![feature(inclusive_range_syntax)]
#![feature(allocator_api)]
#![feature(conservative_impl_trait)]
#![feature(optin_builtin_traits)]
#![feature(type_ascription)]

extern crate jack;
extern crate lazy_ambulance;
use jack::*;
use lazy_ambulance::generator::*;
use lazy_ambulance::ui;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct NH;

impl NotificationHandler for NH {}

struct PH<F, G>
where
    F: Fn(f32) -> G + Send,
    G: Generator,
{
    ports: Vec<Port<AudioOut>>,
    generator_fn: F,
    pitch: Arc<AtomicU32>,
}

impl<F, G> ProcessHandler for PH<F, G>
where
    F: Fn(f32) -> G + Send,
    G: Generator,
{
    fn process(&mut self, client: &Client, scope: &ProcessScope) -> Control {
        let base = f32::from_bits(self.pitch.load(Ordering::Relaxed));
        for sample_index in 0..(scope.n_frames() as usize) {
            let sample = (self.generator_fn)(base).next_sample(client);
            for port in self.ports.iter_mut() {
                let slice = port.as_mut_slice(scope);
                slice[sample_index] = sample;
            }
        }
        Control::Continue
    }
}

fn main_(pitch: Arc<AtomicU32>) -> Result<(), jack::Error> {
    let (client, _status) = Client::new("rusty_client", jack::ClientOptions::NO_START_SERVER)?;
    let mut outputs = vec![];
    outputs.push(client.register_port("output1", AudioOut)?);
    outputs.push(client.register_port("output2", AudioOut)?);

    let _async_client = client.activate_async(
        NH,
        PH {
            ports: outputs,
            generator_fn: generator,
            pitch,
        },
    )?;

    loop {
        std::thread::yield_now();
    }
}

const STEP: f32 = 1.05946309436;

fn generator(base: f32) -> impl Generator {
    Sin::new(base)
    // Add(
    //     Add(note(base), note(base * STEP.powf(7.0))),
    //     note(base * 2.0),
    // )
}

fn note(pitch: f32) -> impl Generator {
    let base = Sin::new(pitch);
    let low_second = Mult(base, 1.0 / 2.0);
    let high_second = Mult(base, 2.0);
    let low_third = Mult(base, 1.0 / 3.0);
    let high_third = Mult(base, 3.0);
    let low_fifth = Mult(base, 1.0 / 5.0);
    let high_fifth = Mult(base, 5.0);
    let low_seventh = Mult(base, 1.0 / 7.0);
    let high_seventh = Mult(base, 7.0);

    Add(
        Add(
            Add(
                Add(
                    Add(
                        Add(Add(Add(base, low_second), high_second), low_third),
                        high_third,
                    ),
                    low_fifth,
                ),
                high_fifth,
            ),
            low_seventh,
        ),
        high_seventh,
    )
}

fn main() {
    let pitch = Arc::new(AtomicU32::new(440f32.to_bits()));
    // ui::pitcher(pitch.clone());
    main_(pitch).unwrap();
}
