use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, SliderView};

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

static notes: [&str; 12] = [
    "A ", "A♯", "B ", "C ", "C♯", "D ", "D♯", "E ", "F ", "F♯", "G ", "G♯"
];

pub fn pitcher(pitch: Arc<AtomicU32>, quit: Sender<bool>) {
    let mut siv = Cursive::new();

    let quit_clone = quit.clone();
    siv.add_global_callback('q', move |s| {
        s.quit();
        quit_clone.send(true);
    });

    siv.add_layer(
        Dialog::around(
            // We give the number of steps in the constructor
            SliderView::horizontal(15)
                // Sets the initial value
                .value(7)
                .on_change(move |s, v| {
                    let title = format!("[ {} ]", notes[(v + notes.len() - 7) % notes.len()]);
                    s.call_on_id("dialog", |view: &mut Dialog| {
                        view.set_title(title)
                    });
                    adjust_pitch(&pitch, v);
                })
                .on_enter(move |s, v| {
                    s.pop_layer();
                    let quit_clone = quit.clone();
                    s.add_layer(
                        Dialog::text(format!("Lucky note {}!", notes[(v + notes.len() - 7) % notes.len()]))
                            .button("Ok", move |s| {
                                s.quit();
                                quit_clone.send(true);
                            }),
                    );
                }),
        ).title(format!("[ {} ]", notes[0]))
            .with_id("dialog"),
    );

    siv.run();
}

fn adjust_pitch(pitch: &AtomicU32, val: usize) {
    let new_pitch = 220_f32 * 1.05946309436_f32.powi(val as i32 - 7);
    pitch.store(new_pitch.to_bits(), Ordering::Relaxed);
}
