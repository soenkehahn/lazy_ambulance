#![feature(allocator_api)]
#![feature(type_ascription)]
#![feature(optin_builtin_traits)]

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
