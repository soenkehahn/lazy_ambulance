use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, SliderView};

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub fn pitcher(pitch: Arc<AtomicU32>) {
    let mut siv = Cursive::new();

    siv.add_global_callback('q', |s| s.quit());

    // Let's add a simple slider in a dialog.
    // Moving the slider will update the dialog's title.
    // And pressing "Enter" will show a new dialog.
    siv.add_layer(
        Dialog::around(
            // We give the number of steps in the constructor
            SliderView::horizontal(15)
                // Sets the initial value
                .value(7)
                .on_change(move |s, v| {
                    let title = format!("[ {} ]", v);
                    s.call_on_id("dialog", |view: &mut Dialog| {
                        view.set_title(title)
                    });
                    adjust_pitch(&pitch, v);
                })
                .on_enter(|s, v| {
                    s.pop_layer();
                    s.add_layer(
                        Dialog::text(format!("Lucky number {}!", v))
                            .button("Ok", Cursive::quit),
                    );
                }),
        ).title("[ 7 ]")
            .with_id("dialog"),
    );

    siv.run();
}

fn adjust_pitch(pitch: &AtomicU32, val: usize) {
    let new_pitch = 440_f32 * 1.05946309436_f32.powi(val as i32 - 7);
    pitch.store(new_pitch.to_bits(), Ordering::Relaxed);
}
