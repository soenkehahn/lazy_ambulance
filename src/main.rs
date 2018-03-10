#![feature(inclusive_range_syntax)]
#![feature(allocator_api)]
#![feature(conservative_impl_trait)]
#![feature(optin_builtin_traits)]
#![feature(type_ascription)]

extern crate jack;
extern crate leaker;
use jack::*;
use leaker::generator::*;

struct NH;

impl NotificationHandler for NH {}

struct PH<G: Generator> {
    ports: Vec<Port<AudioOut>>,
    generator: G,
}

impl<G: Generator> ProcessHandler for PH<G> {
    fn process(&mut self, client: &Client, scope: &ProcessScope) -> Control {
        for sample_index in 0..(scope.n_frames() as usize) {
            let sample = self.generator.next_sample(client);
            for port in self.ports.iter_mut() {
                let slice = port.as_mut_slice(scope);
                slice[sample_index] = sample;
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

const STEP: f32 = 1.05946309436;

fn generator() -> impl Generator {
    let base: f32 = 220.0 * STEP.powf(3.0) / 2.0;
    Add(note(base), note(base * STEP.powf(7.0)))
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
    main_().unwrap();
}
