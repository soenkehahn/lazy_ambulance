#![feature(integer_atomics)]
#![feature(inclusive_range_syntax)]
#![feature(allocator_api)]
#![feature(conservative_impl_trait)]
#![feature(optin_builtin_traits)]
#![feature(type_ascription)]

extern crate jack;
extern crate lazy_ambulance;

use lazy_ambulance::generator::*;
use lazy_ambulance::ui;

use jack::*;

use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

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

fn main_(pitch: Arc<AtomicU32>, quit: Receiver<bool>) -> Result<(), jack::Error> {
    let (client, _status) = Client::new("rusty_client", jack::ClientOptions::NO_START_SERVER)?;
    let mut outputs = vec![];
    outputs.push(client.register_port("output1", AudioOut)?);
    outputs.push(client.register_port("output2", AudioOut)?);

    let generator = Sin::new(Variable::new(pitch));
    let _async_client = client.activate_async(
        NH,
        PH {
            ports: outputs,
            generator,
        },
    )?;

    let _ = quit.recv();

    Ok(())
}

fn main() {
    let pitch = Arc::new(AtomicU32::new(440f32.to_bits()));

    let (quit_tx, quit_rx) = channel::<bool>();

    let pitch_clone = pitch.clone();
    let child = thread::spawn(move || main_(pitch_clone, quit_rx).unwrap());
    thread::sleep(Duration::from_millis(300));
    ui::pitcher(pitch, quit_tx);
    child.join().unwrap();
}
