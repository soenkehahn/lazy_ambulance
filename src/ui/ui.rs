use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, SliderView};

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

static NOTE_NAMES: [&str; 12] = [
    "A ", "A♯", "B ", "C ", "C♯", "D ", "D♯", "E ", "F ", "F♯", "G ", "G♯"
];
const PITCHES: usize = 49;

fn note_name(idx: usize) -> &'static str {
    let nlen = NOTE_NAMES.len() as isize;
    let adj_idx = idx as isize - PITCHES as isize / 2;
    let node_idx = (adj_idx % nlen + nlen) % nlen;

    NOTE_NAMES[node_idx as usize]
}

pub fn pitcher(pitch: Arc<AtomicU32>, quit: Sender<bool>) {
    let mut siv = Cursive::new();

    let quit_clone = quit.clone();
    siv.add_global_callback('q', move |s| {
        s.quit();
        let _ = quit_clone.send(true);
    });

    siv.add_layer(
        Dialog::around(
            // We give the number of steps in the constructor
            SliderView::horizontal(PITCHES)
                // Sets the initial value
                .value(PITCHES/2)
                .on_change(move |s, v| {
                    let title = format!("[ {} ]", note_name(v));
                    s.call_on_id("dialog", |view: &mut Dialog| {
                        view.set_title(title)
                    });
                    adjust_pitch(&pitch, v);
                })
                .on_enter(move |s, v| {
                    s.pop_layer();
                    let quit_clone = quit.clone();
                    s.add_layer(
                        Dialog::text(format!("Lucky note {}!", note_name(v)))
                            .button("Ok", move |s| {
                                s.quit();
                                let _ = quit_clone.send(true);
                            }),
                    );
                }),
        ).title(format!("[ {} ]", note_name(PITCHES / 2)))
            .with_id("dialog"),
    );

    siv.run();
}

// fn note(pitch: f32) -> impl Generator {
//     let base = Sin::new(pitch);
//     let low_second = Mult(base, 1.0 / 2.0);
//     let high_second = Mult(base, 2.0);
//     let low_third = Mult(base, 1.0 / 3.0);
//     let high_third = Mult(base, 3.0);
//     let low_fifth = Mult(base, 1.0 / 5.0);
//     let high_fifth = Mult(base, 5.0);
//     let low_seventh = Mult(base, 1.0 / 7.0);
//     let high_seventh = Mult(base, 7.0);

//     Add(
//         Add(
//             Add(
//                 Add(
//                     Add(
//                         Add(Add(Add(base, low_second), high_second), low_third),
//                         high_third,
//                     ),
//                     low_fifth,
//                 ),
//                 high_fifth,
//             ),
//             low_seventh,
//         ),
//         high_seventh,
//     )
// }

const STEP: f32 = 1.05946309436;

fn adjust_pitch(pitch: &AtomicU32, val: usize) {
    let new_pitch = 440_f32 * STEP.powi(val as i32 - PITCHES as i32 / 2);
    pitch.store(new_pitch.to_bits(), Ordering::Relaxed);
}
