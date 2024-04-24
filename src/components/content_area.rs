use std::sync::Arc;

use freya::prelude::*;

use crate::{core::{pane::Pane, rendering::render_terminal}, events::{Event, Events}};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(pane: Arc<Pane>) -> Element {
    let mut lines = use_signal_sync(|| vec![]);

    use_hook(move || {
        let events = Events::get();
        events.subscribe(move |event| {
            match event {
                Event::OutputUpdate(pane_id) if pane_id == pane.id => {
                    let terminal = pane.terminal()
                        .lock()
                        .expect("Unable to obtain terminal");

                    lines.set(render_terminal(&terminal));
                }
                _ => {}
            }
        });
    });

    rsx!(
        rect {
            padding: "50 50 20 100",
            for line in lines() {
                rect {
                    padding: "2 0",
                    paragraph {
                        for segment in line.segments() {
                            text { "{segment.text}" }
                        }
                    }
                }
            }
        }
    )
}
